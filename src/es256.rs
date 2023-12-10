use crate::base64::base64url_decode;
use crate::encr::{aes_gcm_encrypt, hkdf_simple_expand, hmac_sha256};
use crate::err::{PusherError, Res};
use openssl::bn::{BigNum, BigNumContext};
use openssl::derive::Deriver;
use openssl::ec::{EcGroup, EcKey, EcPoint, PointConversionForm};
use openssl::ecdsa::EcdsaSig;
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private, Public};
use openssl::sha::sha256;

fn get_grp() -> Res<EcGroup> {
    Ok(EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?)
}

#[derive(Debug)]
pub struct Es256Pub {
    key: EcKey<Public>,
}

impl Es256Pub {
    pub fn key_info(&self, user_pub_key: &Self) -> Res<Vec<u8>> {
        let ua_bytes = Vec::try_from(user_pub_key)?;
        let as_bytes = Vec::try_from(self)?;
        Ok([b"WebPush: info\0", ua_bytes.as_slice(), as_bytes.as_slice()].concat())
    }

    pub fn mk_header(&self, salt: &[u8]) -> Res<Vec<u8>> {
        let as_bytes = Vec::try_from(self)?;
        let as_pub_len = as_bytes.len() as u8;
        Ok([
            salt,
            &4096u32.to_be_bytes(),
            &[as_pub_len],
            as_bytes.as_slice(),
        ]
        .concat())
    }
}

impl TryFrom<&[u8]> for Es256Pub {
    type Error = PusherError;

    fn try_from(public_key: &[u8]) -> Res<Self> {
        let grp = get_grp()?;
        let mut ctx = BigNumContext::new()?;
        let pub_point = EcPoint::from_bytes(&grp, &public_key, &mut ctx)?;
        let key = EcKey::from_public_key(&grp, &pub_point)?;
        Ok(Self { key })
    }
}

impl TryFrom<&Es256Pub> for Vec<u8> {
    type Error = PusherError;

    fn try_from(es_pub: &Es256Pub) -> Result<Self, Self::Error> {
        let mut ctx = BigNumContext::new()?;
        let grp = es_pub.key.group();
        let pub_key = es_pub.key.public_key();
        Ok(pub_key.to_bytes(&grp, PointConversionForm::UNCOMPRESSED, &mut ctx)?)
    }
}

#[derive(Debug)]
pub struct Es256 {
    key: EcKey<Private>,
}

impl Es256 {
    pub fn gen() -> Res<Self> {
        let key = EcKey::generate(get_grp()?.as_ref())?;
        Ok(Self { key })
    }

    fn derive_ecdh_secret(&self, peer_pubkey: &Es256Pub) -> Res<Vec<u8>> {
        let peer_pubkey = PKey::public_key_from_pem(&peer_pubkey.key.public_key_to_pem()?)?;
        let pkey = PKey::private_key_from_pem(&self.key.private_key_to_pem()?)?;

        let mut deriver = Deriver::new(&pkey)?;
        deriver.set_peer(&peer_pubkey)?;
        Ok(deriver.derive_to_vec()?)
    }

    fn mk_prk(
        &self,
        peer_pubkey: &Es256Pub,
        auth_secret: &[u8; 16],
        salt: &[u8; 16],
    ) -> Res<Vec<u8>> {
        let self_pub = Es256Pub::try_from(self)?;
        let key_info = self_pub.key_info(peer_pubkey)?;
        let ecdh_secret = self.derive_ecdh_secret(&peer_pubkey)?;
        let prk_key = hmac_sha256(auth_secret, &ecdh_secret)?;
        let ikm = hmac_sha256(&prk_key, &[key_info.as_slice(), &[1]].concat())?;
        hmac_sha256(salt, &ikm)
    }

    pub fn mk_content(
        &self,
        peer_pubkey: &Es256Pub,
        auth_secret: &[u8; 16],
        salt: &[u8; 16],
        plain: &[u8],
    ) -> Res<Vec<u8>> {
        let header = Es256Pub::try_from(self)?.mk_header(salt)?;
        let prk = self.mk_prk(peer_pubkey, auth_secret, salt)?;
        let nonce = hkdf_simple_expand(&prk, b"Content-Encoding: nonce\0\x01")?;
        let cek = hkdf_simple_expand(&prk, b"Content-Encoding: aes128gcm\0\x01")?;
        let (encr, tag) = aes_gcm_encrypt(&[plain, &[2]].concat(), &cek, &nonce)?;
        let encr = [encr, tag.to_vec()].concat();
        Ok([header, encr].concat())
    }

    pub fn sign(&self, data: &[u8]) -> Res<Vec<u8>> {
        let sig = EcdsaSig::sign(&sha256(data), &self.key)?;
        Ok([sig.r().to_vec(), sig.s().to_vec()].concat())
    }

    pub fn verify(&self, data: &[u8], sig: &[u8]) -> Res<bool> {
        let r = BigNum::from_slice(&sig[..32])?;
        let s = BigNum::from_slice(&sig[32..64])?;
        let sig = EcdsaSig::from_private_components(r, s)?;
        Ok(sig.verify(&sha256(data), &self.key)?)
    }

    pub fn private_key(&self) -> Vec<u8> {
        self.key.private_key().to_vec()
    }

    pub fn public_key(&self) -> Res<Vec<u8>> {
        Es256Pub::try_from(self).and_then(|k| Vec::try_from(&k))
    }
}

impl TryFrom<(&[u8], &[u8])> for Es256 {
    type Error = PusherError;

    fn try_from((private_key, public_key): (&[u8], &[u8])) -> Res<Self> {
        let grp = get_grp()?;
        let private_num = BigNum::from_slice(&private_key)?;
        let mut ctx = BigNumContext::new()?;
        let pub_point = EcPoint::from_bytes(&grp, &public_key, &mut ctx)?;
        let key = EcKey::from_private_components(&grp, &private_num, &pub_point)?;
        Ok(Self { key })
    }
}

impl TryFrom<(&str, &str)> for Es256 {
    type Error = PusherError;

    fn try_from((private_b64url, public_b64url): (&str, &str)) -> Res<Self> {
        let private_key = base64url_decode(private_b64url)?;
        let public_key = base64url_decode(public_b64url)?;
        Self::try_from((private_key.as_slice(), public_key.as_slice()))
    }
}

impl TryFrom<&Es256> for Es256Pub {
    type Error = PusherError;
    fn try_from(full: &Es256) -> Res<Self> {
        let key = EcKey::from_public_key(full.key.group(), full.key.public_key())?;
        Ok(Self { key })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vec_from_b64(b64: &str) -> Vec<u8> {
        base64url_decode(b64).unwrap()
    }
    fn es_pub_from_b64(b64: &str) -> Es256Pub {
        vec_from_b64(b64).as_slice().try_into().unwrap()
    }
    fn es_from_b64(b64_priv: &str, b64_pub: &str) -> Es256 {
        let v_priv = vec_from_b64(b64_priv);
        let v_pub = vec_from_b64(b64_pub);
        Es256::try_from((v_priv.as_slice(), v_pub.as_slice())).unwrap()
    }
    fn arr_from_b64<const N: usize>(b64: &str) -> [u8; N] {
        vec_from_b64(b64).as_slice().try_into().unwrap()
    }

    #[test]
    fn key_info_works() {
        // from https://www.rfc-editor.org/rfc/rfc8291#appendix-A
        let as_public = vec_from_b64("BP4z9KsN6nGRTbVYI_c7VJSPQTBtkgcy27mlmlMoZIIgDll6e3vCYLocInmYWAmS6TlzAC8wEqKK6PBru3jl7A8");
        let as_public = Es256Pub::try_from(as_public.as_slice()).unwrap();
        let ua_public = vec_from_b64("BCVxsr7N_eNgVRqvHtD0zTZsEc6-VV-JvLexhqUzORcxaOzi6-AYWXvTBHm4bjyPjs7Vd8pZGH6SRpkNtoIAiw4");
        let ua_public = Es256Pub::try_from(ua_public.as_slice()).unwrap();
        let key_info_exp = vec_from_b64("V2ViUHVzaDogaW5mbwAEJXGyvs3942BVGq8e0PTNNmwRzr5VX4m8t7GGpTM5FzFo7OLr4BhZe9MEebhuPI-OztV3ylkYfpJGmQ22ggCLDgT-M_SrDepxkU21WCP3O1SUj0EwbZIHMtu5pZpTKGSCIA5Zent7wmC6HCJ5mFgJkuk5cwAvMBKiiujwa7t45ewP");
        let key_info = as_public.key_info(&ua_public).unwrap();
        assert_eq!(key_info, key_info_exp);
    }

    #[test]
    fn mk_header_works() {
        let as_public = es_pub_from_b64("BP4z9KsN6nGRTbVYI_c7VJSPQTBtkgcy27mlmlMoZIIgDll6e3vCYLocInmYWAmS6TlzAC8wEqKK6PBru3jl7A8");
        let salt = vec_from_b64("DGv6ra1nlYgDCS1FRnbzlw");
        let header_exp = vec_from_b64("DGv6ra1nlYgDCS1FRnbzlwAAEABBBP4z9KsN6nGRTbVYI_c7VJSPQTBtkgcy27mlmlMoZIIgDll6e3vCYLocInmYWAmS6TlzAC8wEqKK6PBru3jl7A8");
        let header = as_public.mk_header(&salt).unwrap();
        assert_eq!(header, header_exp);
    }

    #[test]
    fn derive_ecdh_secret_works() {
        // from https://www.rfc-editor.org/rfc/rfc8291#appendix-A
        let as_es = es_from_b64(
            "yfWPiYE-n46HLnH0KqZOF1fJJU3MYrct3AELtAQ-oRw",
            "BP4z9KsN6nGRTbVYI_c7VJSPQTBtkgcy27mlmlMoZIIgDll6e3vCYLocInmYWAmS6TlzAC8wEqKK6PBru3jl7A8"
        );
        let ua_public = es_pub_from_b64("BCVxsr7N_eNgVRqvHtD0zTZsEc6-VV-JvLexhqUzORcxaOzi6-AYWXvTBHm4bjyPjs7Vd8pZGH6SRpkNtoIAiw4");

        let ecdh_secret_exp = vec_from_b64("kyrL1jIIOHEzg3sM2ZWRHDRB62YACZhhSlknJ672kSs");
        let ecdh_secret = as_es.derive_ecdh_secret(&ua_public).unwrap();
        assert_eq!(ecdh_secret, ecdh_secret_exp);
    }

    #[test]
    fn prk_works() {
        // from https://www.rfc-editor.org/rfc/rfc8291#appendix-A
        let as_es = es_from_b64(
            "yfWPiYE-n46HLnH0KqZOF1fJJU3MYrct3AELtAQ-oRw",
            "BP4z9KsN6nGRTbVYI_c7VJSPQTBtkgcy27mlmlMoZIIgDll6e3vCYLocInmYWAmS6TlzAC8wEqKK6PBru3jl7A8"
        );
        let ua_public = es_pub_from_b64("BCVxsr7N_eNgVRqvHtD0zTZsEc6-VV-JvLexhqUzORcxaOzi6-AYWXvTBHm4bjyPjs7Vd8pZGH6SRpkNtoIAiw4");
        let salt = arr_from_b64("DGv6ra1nlYgDCS1FRnbzlw");
        let auth_secret = arr_from_b64("BTBZMqHH6r4Tts7J_aSIgg");

        let prk_exp = vec_from_b64("09_eUZGrsvxChDCGRCdkLiDXrReGOEVeSCdCcPBSJSc");
        let prk = as_es.mk_prk(&ua_public, &auth_secret, &salt).unwrap();
        assert_eq!(prk, prk_exp);
    }

    #[test]
    fn mk_content_works() {
        let as_es = es_from_b64(
            "yfWPiYE-n46HLnH0KqZOF1fJJU3MYrct3AELtAQ-oRw",
            "BP4z9KsN6nGRTbVYI_c7VJSPQTBtkgcy27mlmlMoZIIgDll6e3vCYLocInmYWAmS6TlzAC8wEqKK6PBru3jl7A8"
        );
        let ua_public = es_pub_from_b64("BCVxsr7N_eNgVRqvHtD0zTZsEc6-VV-JvLexhqUzORcxaOzi6-AYWXvTBHm4bjyPjs7Vd8pZGH6SRpkNtoIAiw4");
        let salt = arr_from_b64("DGv6ra1nlYgDCS1FRnbzlw");
        let auth = arr_from_b64("BTBZMqHH6r4Tts7J_aSIgg");
        let plain = vec_from_b64("V2hlbiBJIGdyb3cgdXAsIEkgd2FudCB0byBiZSBhIHdhdGVybWVsb24");

        let content = as_es.mk_content(&ua_public, &auth, &salt, &plain).unwrap();
        let content_exp = vec_from_b64("DGv6ra1nlYgDCS1FRnbzlwAAEABBBP4z9KsN6nGRTbVYI_c7VJSPQTBtkgcy27mlmlMoZIIgDll6e3vCYLocInmYWAmS6TlzAC8wEqKK6PBru3jl7A_yl95bQpu6cVPTpK4Mqgkf1CXztLVBSt2Ks3oZwbuwXPXLWyouBWLVWGNWQexSgSxsj_Qulcy4a-fN");
        assert_eq!(content.len(), 144);
        assert_eq!(content, content_exp);
    }

    #[test]
    fn signatures_are_verified() {
        let key = Es256::gen().unwrap();
        let data = b"this is some data";

        let mut sig = key.sign(data).unwrap();
        assert!(key.verify(data, &sig).unwrap());

        sig[0] ^= 17;
        assert!(!key.verify(data, &sig).unwrap());
    }
}
