use crate::err::Res;
use openssl::base64::{decode_block, encode_block};

pub fn base64url_encode<V: AsRef<[u8]>>(plain: V) -> String {
    encode_block(plain.as_ref())
        .replace('+', "-")
        .replace('/', "_")
        .trim_end_matches('=')
        .to_string()
}

pub fn base64url_decode<S: AsRef<str>>(encoded: S) -> Res<Vec<u8>> {
    let mut unpadded = encoded.as_ref().replace('-', "+").replace('_', "/");
    while unpadded.len() % 4 != 0 {
        unpadded.push('=')
    }
    Ok(decode_block(&unpadded)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64url_encode_matches_rfc8291_example() {
        // from https://www.rfc-editor.org/rfc/rfc8291#section-5
        let plaintext = b"When I grow up, I want to be a watermelon";
        let encoded = "V2hlbiBJIGdyb3cgdXAsIEkgd2FudCB0byBiZSBhIHdhdGVybWVsb24";
        assert_eq!(base64url_encode(plaintext), encoded);
    }

    #[test]
    fn base64url_decode_is_inverse_of_encode() {
        let plaintext = b"encode and decode are inverses of each other";
        let encoded = base64url_encode(plaintext);
        let decoded = base64url_decode(encoded);
        assert_eq!(plaintext, decoded.unwrap().as_slice());
    }

    #[test]
    fn base64url_decode_might_fail() {
        let plaintext = "***";
        assert!(base64url_decode(plaintext).is_err());
    }
}
