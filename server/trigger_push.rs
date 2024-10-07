use axum::extract::State;
use pusher::err::Res;
use reqwest::StatusCode;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;

pub async fn write_to_socket(State(push_test_addr): State<Option<Arc<str>>>) -> Res<StatusCode> {
    let Some(push_test_addr) = push_test_addr else {
        tracing::info!("Trying to write without socket addr");
        return Ok(StatusCode::OK);
    };
    let mut stream = UnixStream::connect(push_test_addr.as_ref()).await?;

    let current_ts = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let msg = format!("test at {}", current_ts.as_secs());

    stream.write_all(msg.as_bytes()).await?;
    tracing::info!("Wrote to {}", push_test_addr);

    Ok(StatusCode::OK)
}
