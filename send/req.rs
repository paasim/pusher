use pusher::base64::base64url_encode;
use pusher::db::get_pool;
use pusher::encr::gen_salt;
use pusher::err::Result;
use pusher::err_other;
use pusher::es256::Es256;
use pusher::jwt::mk_vapid_jwt;
use pusher::subscription::{get_subscriptions, Subscription};
use pusher::utils::get_var;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE};
use reqwest::{Client, Response, StatusCode};
use url::Url;

pub struct VapidConfig {
    key: Es256,
    subject: Url,
}

impl VapidConfig {
    pub fn from_env() -> Result<Self> {
        let public_key = get_var("VAPID_PUBLIC_KEY")?;
        let private_key = get_var("VAPID_PRIVATE_KEY")?;
        let subject = err_other!(Url::parse(&get_var("VAPID_SUBJECT")?))?;
        let key = Es256::try_from((private_key.as_str(), public_key.as_str()))?;
        Ok(Self { key, subject })
    }

    pub fn public_key(&self) -> Result<String> {
        self.key.public_key().map(base64url_encode)
    }
}

fn construct_headers(
    jwt: &str,
    k: &str,
    vapid_pub: &str,
    len: usize,
    ttl: usize,
) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    let auth = format!("vapid t={}, k={}", jwt, k);
    headers.insert(AUTHORIZATION, auth.try_into()?);
    headers.insert("Crypto-Key", format!("p256ecdsa={}", vapid_pub).try_into()?);
    headers.insert(CONTENT_LENGTH, len.into());
    headers.insert(CONTENT_TYPE, "application/octet-stream".try_into()?);
    headers.insert(CONTENT_ENCODING, "aes128gcm".try_into()?);
    headers.insert("TTL", ttl.into());
    Ok(headers)
}

pub async fn send_notification(
    sub: &Subscription,
    vapid: &VapidConfig,
    content: &[u8],
    ttl: usize,
) -> Result<Response> {
    let aud = Url::parse(&sub.endpoint().origin().ascii_serialization())?;
    let (jwt, k) = mk_vapid_jwt(&aud, &vapid.subject, 10, &vapid.key)?;

    let local_key = Es256::gen()?;
    let salt = gen_salt::<16>()?;
    let payload = local_key.mk_content(sub.p256dh(), sub.auth(), &salt, content)?;

    let headers = construct_headers(&jwt, &k, &vapid.public_key()?, payload.len(), ttl)?;
    let req = Client::new()
        .post(sub.endpoint().clone())
        .body(payload)
        .headers(headers);

    Ok(req.send().await?)
}

type Resp = reqwest::Result<String>;

#[tokio::main]
pub async fn send_notifications(
    database_url: &str,
    vapid: &VapidConfig,
    content: &[u8],
    ttl: usize,
    encryption_key: [u8; 16],
) -> Result<Vec<(Url, StatusCode, Resp)>> {
    let pool = get_pool(database_url, true)?;
    let mut res = vec![];
    for sub in get_subscriptions(pool, encryption_key).await? {
        let resp = send_notification(&sub, vapid, content, ttl).await?;
        res.push((sub.endpoint().clone(), resp.status(), resp.text().await));
    }
    Ok(res)
}
