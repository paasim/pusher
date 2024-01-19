use crate::base64::base64url_decode;
use crate::encr::{aes_gcm_decrypt, aes_gcm_encrypt, gen_salt};
use crate::err::Res;
use crate::es256::Es256Pub;
use crate::utils::to_array;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::de::Error;
use serde::Deserialize;
use sqlx::{query, query_as, SqlitePool};
use url::Url;

#[derive(Debug)]
pub struct Subscription {
    endpoint: Url,
    expiration_time: Option<u32>,
    auth: [u8; 16],
    p256dh: Es256Pub,
}

impl<'de> Deserialize<'de> for Subscription {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SubscriptionKeysRaw {
            auth: String,
            p256dh: String,
        }
        #[derive(Deserialize)]
        pub struct SubscriptionRaw {
            endpoint: String,
            #[serde(rename = "expirationTime")]
            expiration_time: Option<u32>,
            keys: SubscriptionKeysRaw,
        }
        let raw = SubscriptionRaw::deserialize(deserializer)?;
        let auth = base64url_decode(raw.keys.auth).and_then(to_array);
        let p256dh =
            base64url_decode(raw.keys.p256dh).and_then(|k| Es256Pub::try_from(k.as_slice()));
        Ok(Subscription {
            endpoint: Url::parse(&raw.endpoint).map_err(D::Error::custom)?,
            expiration_time: raw.expiration_time,
            auth: auth.map_err(D::Error::custom)?,
            p256dh: p256dh.map_err(D::Error::custom)?,
        })
    }
}

impl Subscription {
    pub fn encrypted_auth(&self, encrytion_key: &[u8; 16]) -> Res<([u8; 12], Vec<u8>, [u8; 16])> {
        let salt = gen_salt()?;
        let (auth_encr, tag) = aes_gcm_encrypt(&self.auth, encrytion_key, &salt)?;
        Ok((salt, auth_encr, tag))
    }
    pub fn endpoint(&self) -> &Url {
        &self.endpoint
    }

    pub fn auth(&self) -> &[u8; 16] {
        &self.auth
    }

    pub fn p256dh(&self) -> &Es256Pub {
        &self.p256dh
    }
}

pub async fn subscribe(
    State((pool, encryption_key)): State<(SqlitePool, [u8; 16])>,
    Json(sub): Json<Subscription>,
) -> Res<StatusCode> {
    tracing::info!("SUBSCRIBE {}", sub.endpoint());
    insert_subscription(&pool, &encryption_key, &sub).await?;
    Ok(StatusCode::OK)
}

pub async fn unsubscribe(
    State((pool, _)): State<(SqlitePool, [u8; 16])>,
    Json(sub): Json<Subscription>,
) -> Res<StatusCode> {
    tracing::info!("UNSUBSCRIBE {}", sub.endpoint());
    delete_subscription(&pool, sub.endpoint()).await?;
    Ok(StatusCode::OK)
}

#[derive(Debug, Deserialize)]
pub struct SubscriptionRow {
    #[allow(dead_code)]
    id: u32,
    endpoint: String,
    expiration_time: Option<u32>,
    auth_encr: Vec<u8>,
    tag: Vec<u8>,
    salt: Vec<u8>,
    p256dh: Vec<u8>,
}

impl SubscriptionRow {
    pub fn decrypt(self, decryption_key: &[u8; 16]) -> Res<Subscription> {
        let tag = to_array(self.tag)?;
        let auth_decr = aes_gcm_decrypt(&self.auth_encr, decryption_key, &self.salt, &tag)?;
        Ok(Subscription {
            endpoint: Url::parse(&self.endpoint)?,
            expiration_time: self.expiration_time,
            auth: to_array(auth_decr)?,
            p256dh: Es256Pub::try_from(self.p256dh.as_slice())?,
        })
    }
}

pub async fn delete_subscription(pool: &SqlitePool, endpoint: &Url) -> Res<u32> {
    let endpoint = endpoint.to_string();
    let id_row = query!(
        r#"
        DELETE FROM subscription WHERE endpoint = ?
        RETURNING id AS "id: u32""#,
        endpoint
    )
    .fetch_one(pool)
    .await?;
    Ok(id_row.id)
}

pub async fn insert_subscription(
    pool: &SqlitePool,
    encryption_key: &[u8; 16],
    sub: &Subscription,
) -> Res<u32> {
    let (salt, auth_encr, tag) = sub.encrypted_auth(encryption_key)?;
    let p256dh = Vec::try_from(&sub.p256dh)?;
    let salt = salt.as_slice();
    let tag = tag.as_slice();
    let endpoint = sub.endpoint.to_string();
    let id_row = query!(
        r#"
        INSERT INTO subscription
            (endpoint, expiration_time, auth_encr, tag, salt, p256dh)
        VALUES
            (?, ?, ?, ?, ?, ?)
        RETURNING id AS "id: u32"
        "#,
        endpoint,
        sub.expiration_time,
        auth_encr,
        tag,
        salt,
        p256dh,
    )
    .fetch_one(pool)
    .await?;
    Ok(id_row.id)
}

pub async fn get_subscriptions(pool: &SqlitePool, key: &[u8; 16]) -> Res<Vec<Subscription>> {
    query_as!(
        SubscriptionRow,
        r#"
        SELECT
            id AS "id: u32",
            endpoint,
            expiration_time AS "expiration_time: u32",
            auth_encr,
            tag,
            salt,
            p256dh
        FROM subscription
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| r.decrypt(key))
    .collect()
}
