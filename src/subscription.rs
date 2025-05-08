use crate::base64::base64url_decode;
use crate::encr::{aes_gcm_decrypt, aes_gcm_encrypt, gen_salt};
use crate::err::Result;
use crate::es256::Es256Pub;
use crate::utils::to_array;
use crate::{err_other, err_to_resp};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use deadpool_sqlite::rusqlite::Connection;
use deadpool_sqlite::Pool;
use serde::de::Error;
use serde::Deserialize;
use url::Url;

#[derive(Debug)]
pub struct Subscription {
    endpoint: Url,
    name: String,
    expiration_time: Option<u32>,
    auth: [u8; 16],
    p256dh: Es256Pub,
}

#[derive(Debug, Deserialize)]
pub struct Endpoint {
    endpoint: Url,
}

impl<'de> Deserialize<'de> for Subscription {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
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
            endpoint: Url,
            name: String,
            #[serde(rename = "expirationTime")]
            expiration_time: Option<u32>,
            keys: SubscriptionKeysRaw,
        }
        let raw = SubscriptionRaw::deserialize(deserializer)?;
        let auth = base64url_decode(raw.keys.auth).and_then(to_array);
        let p256dh =
            base64url_decode(raw.keys.p256dh).and_then(|k| Es256Pub::try_from(k.as_slice()));
        Ok(Self {
            endpoint: raw.endpoint,
            name: raw.name,
            expiration_time: raw.expiration_time,
            auth: auth.map_err(D::Error::custom)?,
            p256dh: p256dh.map_err(D::Error::custom)?,
        })
    }
}

impl Subscription {
    pub fn encrypted_auth(
        &self,
        encrytion_key: &[u8; 16],
    ) -> Result<([u8; 12], Vec<u8>, [u8; 16])> {
        let salt = gen_salt()?;
        let (auth_encr, tag) = aes_gcm_encrypt(&self.auth, encrytion_key, &salt)?;
        Ok((salt, auth_encr, tag))
    }

    pub fn endpoint(&self) -> &Url {
        &self.endpoint
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn auth(&self) -> &[u8; 16] {
        &self.auth
    }

    pub fn p256dh(&self) -> &Es256Pub {
        &self.p256dh
    }

    pub fn query(conn: &Connection, key: [u8; 16]) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT endpoint, name, expiration_time, auth_encr, salt, tag, p256dh FROM subscription",
        )?;
        let mut rows = stmt.query([])?;
        let mut v = vec![];
        while let Some(r) = rows.next()? {
            let auth_decr = aes_gcm_decrypt(&r.get::<_, Vec<_>>(3)?, &key, &r.get(4)?, &r.get(5)?)?;
            v.push(Self {
                endpoint: err_other!(Url::parse(&r.get::<_, String>(0)?))?,
                name: r.get(1)?,
                expiration_time: r.get(2)?,
                auth: to_array(auth_decr)?,
                p256dh: Es256Pub::try_from(r.get::<_, Vec<_>>(6)?.as_slice())?,
            });
        }
        Ok(v)
    }
}

pub async fn subscribe(
    State((pool, encryption_key)): State<(Pool, [u8; 16])>,
    Json(sub): Json<Subscription>,
) -> Response {
    tracing::info!("SUBSCRIBE {}", sub.endpoint());
    err_to_resp!(insert_subscription(pool, &encryption_key, &sub).await);
    StatusCode::OK.into_response()
}

pub async fn unsubscribe(
    State((pool, _)): State<(Pool, [u8; 16])>,
    Query(query): Query<Endpoint>,
) -> Response {
    tracing::info!("UNSUBSCRIBE {}", query.endpoint);
    err_to_resp!(delete_subscription(pool, &query.endpoint).await);
    StatusCode::OK.into_response()
}

pub async fn delete_subscription(pool: Pool, endpoint: &Url) -> Result<u32> {
    let conn = pool.get().await?;
    let ep = endpoint.to_string();
    conn.interact(move |c| {
        Ok(c.query_row(
            "DELETE FROM subscription WHERE endpoint = (?1) RETURNING id",
            [ep],
            |r| r.get(0),
        )?)
    })
    .await?
}

pub async fn insert_subscription(
    pool: Pool,
    encryption_key: &[u8; 16],
    sub: &Subscription,
) -> Result<u32> {
    let (salt, auth_encr, tag) = sub.encrypted_auth(encryption_key)?;
    let p256dh = Vec::try_from(&sub.p256dh)?;
    let endpoint = sub.endpoint.to_string();
    let name = sub.name.clone();
    let expr = sub.expiration_time;
    let conn = pool.get().await?;
    conn.interact(move |c| {
        Ok(c.query_row(
            "INSERT INTO subscription
            (endpoint, name, expiration_time, auth_encr, tag, salt, p256dh)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            RETURNING id",
            (endpoint, name, expr, auth_encr, tag, salt, p256dh),
            |r| r.get(0),
        )?)
    })
    .await?
}

pub async fn get_subscriptions(pool: &Pool, key: [u8; 16]) -> Result<Vec<Subscription>> {
    let conn = pool.get().await?;
    conn.interact(move |c| Subscription::query(c, key)).await?
}
