use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use pusher::base64::{base64url_decode, base64url_encode};
use pusher::err::{Error, Result};
use pusher::es256::Es256Pub;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct PublicKey {
    vapid_public_key: String,
}

impl TryFrom<&str> for PublicKey {
    type Error = Error;

    /// Assumes the `k`-part of the public key is [base64url_encode]d
    /// as described in rfc8292 section 3.2.
    fn try_from(key: &str) -> Result<Self> {
        let vapid_public_key = base64url_decode(key)
            .and_then(|k| Es256Pub::try_from(k.as_slice()))
            .and_then(|k| Vec::try_from(&k))
            .map(base64url_encode)?;
        Ok(Self { vapid_public_key })
    }
}

pub fn router() -> Router<PublicKey> {
    Router::new().route("/pubkey", get(get_pubkey))
}

/// Return the server (VAPID) public key
async fn get_pubkey(State(pubkey): State<PublicKey>) -> (StatusCode, Json<PublicKey>) {
    (StatusCode::OK, Json(pubkey))
}
