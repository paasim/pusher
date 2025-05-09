use crate::msg::Msg;
use crate::req::send_notifications;
use crate::{Config, Mode};
use pusher::db::get_pool;
use pusher::err::Result;
use std::path::Path;
use tokio::fs;
use tokio::net::UnixListener;
use tracing::Level;

async fn get_listener(path: &Path) -> Result<UnixListener> {
    if fs::try_exists(path).await? {
        fs::remove_file(path).await?;
    }
    Ok(UnixListener::bind(path)?)
}

/// Listen for connections to the socket specified in [Config] and forward the socket
/// input as a push message to all subscribed clients.
pub async fn listen(config: Config) -> Result<()> {
    let listener = get_listener(&config.push_test_addr).await?;
    let pool = get_pool(&config.db_path, true)?;
    let mut i = 0;
    while let Ok((stream, _addr)) = listener.accept().await {
        let span = tracing::span!(Level::INFO, "msg_ind", i);
        let _enter = span.enter();
        let content = Msg::from_stream(stream, config.title.clone())
            .await
            .and_then(Vec::try_from)?;
        send_notifications(
            &pool,
            &config.vapid_conf,
            &content,
            10,
            config.encryption_key,
        )
        .await?;
        i += 1;
    }
    Ok(())
}

pub async fn msg_from_stdin(config: Config) -> Result<()> {
    let pool = get_pool(&config.db_path, true)?;
    let content = Msg::from_stdin(config.title).and_then(Vec::try_from)?;
    send_notifications(
        &pool,
        &config.vapid_conf,
        &content,
        10,
        config.encryption_key,
    )
    .await?;
    Ok(())
}

#[tokio::main]
pub async fn run(config: Config) -> Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_max_level(Level::INFO)
        .init();
    match config.mode {
        Mode::Server => listen(config).await,
        Mode::Single => msg_from_stdin(config).await,
    }
}
