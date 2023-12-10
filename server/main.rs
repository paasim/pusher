use crate::vapid::PublicKey;
use pusher::base64::base64url_decode;
use pusher::err::Res;
use pusher::utils::{get_var, to_array};
use std::net::SocketAddr;

mod server;
mod vapid;

fn get_conf() -> Res<(PublicKey, SocketAddr, [u8; 16], String)> {
    let pubkey = PublicKey::try_from(get_var("VAPID_PUBLIC_KEY")?.as_str())?;
    let port = get_var("PORT")?.parse()?;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let encryption_key = get_var("DATABASE_ENCRYPTION_KEY")
        .and_then(base64url_decode)
        .and_then(to_array)?;
    let db_url = get_var("DATABASE_URL")?;
    Ok((pubkey, addr, encryption_key, db_url))
}

fn main() -> Res<()> {
    let (pubkey, addr, encryption_key, db_url) = get_conf().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1)
    });

    server::run(pubkey, addr, encryption_key, &db_url)
}
