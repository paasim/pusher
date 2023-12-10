use crate::err::Res;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{migrate, SqlitePool};
use std::str::FromStr;

pub async fn get_pool(db_url: &str, read_only: bool) -> Res<SqlitePool> {
    let opt = SqliteConnectOptions::from_str(db_url)?.read_only(read_only);
    let pool = SqlitePool::connect_with(opt).await?;

    migrate!().run(&pool).await?;
    Ok(pool)
}
