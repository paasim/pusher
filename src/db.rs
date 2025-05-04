use crate::err::Res;
use deadpool_sqlite::{Config, Pool, Runtime};

pub fn get_pool(db_path: &str, read_only: bool) -> Res<Pool> {
    match read_only {
        true => Ok(Config::new(format!("file:{db_path}?mode=ro")).create_pool(Runtime::Tokio1)?),
        false => Ok(Config::new(db_path).create_pool(Runtime::Tokio1)?),
    }
}
