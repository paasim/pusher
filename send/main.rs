use pusher::base64::base64url_decode;
use pusher::err::Result;
use pusher::err_other;
use pusher::utils::{get_var, to_array};
use req::VapidConfig;
use server::run;
use std::env;
use std::path::PathBuf;

mod msg;
mod req;
mod server;

pub struct Config {
    pub title: String,
    pub encryption_key: [u8; 16],
    pub db_path: String,
    pub vapid_conf: VapidConfig,
    pub push_test_addr: PathBuf,
    pub mode: Mode,
}

pub enum Mode {
    Server,
    Single,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let mut args = env::args();
        let progname = args.next().ok_or("invalid args")?;
        let (mode, title) = match (args.next(), args.next()) {
            (Some(s0), Some(s1)) if s0 == "--server" => (Mode::Server, s1),
            (Some(s0), Some(s1)) if s1 == "--server" => (Mode::Server, s0),
            (Some(s), None) => (Mode::Single, s),
            _ => return Err(format!("usage: {progname} [--server] title").into()),
        };
        let encryption_key = get_var("DATABASE_ENCRYPTION_KEY")
            .and_then(base64url_decode)
            .and_then(to_array)?;
        let db_path = get_var("DATABASE_PATH")?;
        let push_test_addr = err_other!(
            get_var("PUSH_SOCKET_ADDR")?.parse(),
            "invalid PUSH_SOCKET_ADDR"
        )?;
        let vapid_conf = VapidConfig::from_env()?;
        Ok(Self {
            title,
            encryption_key,
            db_path,
            vapid_conf,
            push_test_addr,
            mode,
        })
    }
}

fn main() {
    if let Err(e) = Config::from_env().and_then(run) {
        eprintln!("{e}");
        std::process::exit(1)
    };
}
