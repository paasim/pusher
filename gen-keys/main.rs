use pusher::base64::base64url_encode;
use pusher::encr::gen_salt;
use pusher::err::Result;
use pusher::es256::Es256;

fn get_conf() -> Result<(Vec<u8>, Vec<u8>, [u8; 16])> {
    let key = Es256::gen()?;
    Ok((key.public_key()?, key.private_key(), gen_salt::<16>()?))
}

fn main() {
    let (pub_key, priv_key, encr_key) = get_conf().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1)
    });

    println!("VAPID_PUBLIC_KEY={}", base64url_encode(pub_key));
    println!("VAPID_PRIVATE_KEY={}", base64url_encode(priv_key));
    println!("DATABASE_ENCRYPTION_KEY={}", base64url_encode(encr_key));
}
