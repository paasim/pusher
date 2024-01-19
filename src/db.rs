use crate::err::Res;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{migrate, SqlitePool};

pub async fn get_pool(db_path: &str, read_only: bool) -> Res<SqlitePool> {
    let opt = SqliteConnectOptions::new()
        .filename(db_path)
        .read_only(read_only)
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(opt).await?;

    migrate!().run(&pool).await?;
    Ok(pool)
}
