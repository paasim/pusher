use crate::vapid::PublicKey;
use pusher::base64::base64url_decode;
use pusher::err::Res;
use pusher::utils::{get_var, to_array};
use std::net::SocketAddr;

mod server;
mod trigger_push;
mod vapid;

pub struct Config {
    pub pubkey: PublicKey,
    pub listen_addr: SocketAddr,
    pub encryption_key: [u8; 16],
    pub db_path: String,
    pub push_test_addr: Option<String>,
}

impl Config {
    pub fn from_env() -> Res<Self> {
        let pubkey = PublicKey::try_from(get_var("VAPID_PUBLIC_KEY")?.as_str())?;
        let port = get_var("PORT")?.parse()?;
        let listen_addr = SocketAddr::from(([127, 0, 0, 1], port));
        let encryption_key = get_var("DATABASE_ENCRYPTION_KEY")
            .and_then(base64url_decode)
            .and_then(to_array)?;
        let db_path = get_var("DATABASE_PATH")?;
        let push_test_addr = get_var("PUSH_TEST_ADDR").ok();
        Ok(Self {
            pubkey,
            listen_addr,
            encryption_key,
            db_path,
            push_test_addr,
        })
    }
}

fn main() -> Res<()> {
    let conf = Config::from_env().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1)
    });
    server::run(conf)
}
