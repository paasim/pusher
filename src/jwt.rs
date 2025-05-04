use crate::err::Result;
use crate::es256::Es256;
use crate::{base64::base64url_encode, err_other};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct JwtHeader {
    alg: String,
    typ: String,
}

impl JwtHeader {
    fn es_256() -> Self {
        Self {
            typ: String::from("JWT"),
            alg: String::from("ES256"),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct JwtPayload {
    aud: String,
    exp: u32,
    sub: String,
}

fn mk_jwt_data(
    push_resource: &Url,
    sub: &Url,
    ttl_minutes: u32,
) -> Result<(JwtHeader, JwtPayload)> {
    let header = JwtHeader::es_256();
    let time = err_other!(SystemTime::now().duration_since(UNIX_EPOCH))?;
    let payload = JwtPayload {
        aud: push_resource.origin().ascii_serialization(),
        exp: time.as_secs() as u32 + ttl_minutes * 60,
        sub: sub.to_string(),
    };

    Ok((header, payload))
}

fn to_signed_jwt(info: &JwtHeader, payload: &JwtPayload, key: &Es256) -> Result<String> {
    let header = serde_json::to_string(info).map(|s| base64url_encode(s.as_bytes()))?;
    let payload = serde_json::to_string(payload).map(|p| base64url_encode(p.as_bytes()))?;
    let data = [header, payload].join(".");
    let sig = key.sign(data.as_bytes())?;
    Ok([data, base64url_encode(sig)].join("."))
}

pub fn mk_jwt(push_resource: &Url, subject: &Url, ttl_minutes: u32, key: &Es256) -> Result<String> {
    let (header, payload) = mk_jwt_data(push_resource, subject, ttl_minutes)?;
    to_signed_jwt(&header, &payload, key)
}

pub fn mk_vapid_jwt(
    push_resource: &Url,
    subject: &Url,
    ttl_minutes: u32,
    key: &Es256,
) -> Result<(String, String)> {
    let jwt = mk_jwt(push_resource, subject, ttl_minutes, key)?;
    let k = base64url_encode(key.public_key()?);
    Ok((jwt, k))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base64::base64url_decode;
    use serde::de::DeserializeOwned;

    fn from_b64_json<T: DeserializeOwned>(encoded: &str) -> T {
        let decoded = base64url_decode(encoded).unwrap();
        let json_str = std::str::from_utf8(&decoded).unwrap();
        serde_json::from_str(json_str).unwrap()
    }

    #[test]
    fn to_signed_jwt_works() {
        let key = Es256::gen().unwrap();
        let (header, payload) = mk_jwt_data(
            &Url::parse("http://www.www.www").unwrap(),
            &Url::parse("mailto:test@email.test").unwrap(),
            1,
        )
        .unwrap();
        let jwt = to_signed_jwt(&header, &payload, &key).unwrap();

        let jwt_components: Vec<&str> = jwt.split('.').collect();
        assert_eq!(jwt_components.len(), 3);

        let header_parsed: JwtHeader = from_b64_json(jwt_components[0]);
        assert_eq!(header, header_parsed);

        let payload_parsed: JwtPayload = from_b64_json(jwt_components[1]);
        assert_eq!(payload, payload_parsed);

        let sig = base64url_decode(jwt_components[2]).unwrap();
        let data = jwt_components[..2].join(".");
        assert!(key.verify(data.as_bytes(), &sig).unwrap())
    }
}
