use crate::err::Res;
use crate::utils::to_array;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::rand::rand_bytes;
use openssl::sign::Signer;
use openssl::symm::{decrypt_aead, encrypt_aead, Cipher};

// FIXME: could this somehow sometimes produce 64
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Res<Vec<u8>> {
    let key = PKey::hmac(key)?;
    let dig = Signer::new(MessageDigest::sha256(), &key)?.sign_oneshot_to_vec(data)?;
    Ok(dig)
}

/// assumes that one round is enough and 1 is appended to info
pub fn hkdf_simple_expand<const N: usize>(prk: &[u8], info: &[u8]) -> Res<[u8; N]> {
    to_array(hmac_sha256(prk, info)?)
}

pub fn gen_salt<const N: usize>() -> Res<[u8; N]> {
    let mut buf = [0; N];
    rand_bytes(&mut buf)?;
    Ok(buf)
}

pub fn aes_gcm_encrypt(data: &[u8], key: &[u8; 16], iv: &[u8; 12]) -> Res<(Vec<u8>, [u8; 16])> {
    let mut tag = [0; 16];
    let encr = encrypt_aead(Cipher::aes_128_gcm(), key, Some(iv), &[], data, &mut tag)?;
    Ok((encr, tag))
}

pub fn aes_gcm_decrypt(data: &[u8], key: &[u8], iv: &[u8], tag: &[u8; 16]) -> Res<Vec<u8>> {
    let decr = decrypt_aead(Cipher::aes_128_gcm(), key, Some(iv), &[], data, tag)?;
    Ok(decr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base64::{base64url_decode, base64url_encode};

    fn vec_from_b64(b64: &str) -> Vec<u8> {
        base64url_decode(b64).unwrap()
    }
    fn arr_from_b64<const N: usize>(b64: &str) -> [u8; N] {
        vec_from_b64(b64).as_slice().try_into().unwrap()
    }

    #[test]
    fn encr_decr_are_inverses() {
        let data = b"encryption and decryption are inverses of each other";
        let key = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let iv = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let (encr, tag) = aes_gcm_encrypt(data, key, iv).unwrap();
        let decr = aes_gcm_decrypt(&encr, key, iv, &tag).unwrap();
        assert_eq!(data, decr.as_slice());
    }

    #[test]
    fn hkdf_simple_expand_works() {
        // from https://www.rfc-editor.org/rfc/rfc8291#appendix-A
        let prk = vec_from_b64("09_eUZGrsvxChDCGRCdkLiDXrReGOEVeSCdCcPBSJSc");

        let nonce = hkdf_simple_expand::<12>(&prk, b"Content-Encoding: nonce\0\x01").unwrap();
        let nonce_exp = vec_from_b64("4h_95klXJ5E_qnoN");
        assert_eq!(&nonce, nonce_exp.as_slice());

        let cek = hkdf_simple_expand::<16>(&prk, b"Content-Encoding: aes128gcm\0\x01").unwrap();
        let cek_exp = vec_from_b64("oIhVW04MRdy2XN9CiKLxTg");
        assert_eq!(&cek, cek_exp.as_slice());
    }

    #[test]
    fn encryption_works() {
        // from https://www.rfc-editor.org/rfc/rfc8291#appendix-A
        let mut plain = vec_from_b64("V2hlbiBJIGdyb3cgdXAsIEkgd2FudCB0byBiZSBhIHdhdGVybWVsb24");
        plain.push(2);
        let nonce = arr_from_b64("4h_95klXJ5E_qnoN");
        let cek = arr_from_b64("oIhVW04MRdy2XN9CiKLxTg");

        let encr_exp =
            "8pfeW0KbunFT06SuDKoJH9Ql87S1QUrdirN6GcG7sFz1y1sqLgVi1VhjVkHsUoEsbI_0LpXMuGvnzQ";
        let (encr, tag) = aes_gcm_encrypt(&plain, &cek, &nonce).unwrap();

        let encr = [encr, tag.to_vec()].concat();
        assert_eq!(base64url_encode(encr), encr_exp);
    }
}
