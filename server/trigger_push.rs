use axum::extract::State;
use axum::response::{IntoResponse, Response};
use pusher::err_to_resp;
use reqwest::StatusCode;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;

pub async fn write_to_socket(State(push_test_addr): State<Option<Arc<str>>>) -> Response {
    let Some(push_test_addr) = push_test_addr else {
        tracing::info!("Trying to write without socket addr");
        return StatusCode::OK.into_response();
    };
    let mut stream = err_to_resp!(UnixStream::connect(push_test_addr.as_ref()).await);

    let current_ts = err_to_resp!(SystemTime::now().duration_since(UNIX_EPOCH));
    let msg = format!("test at {}", current_ts.as_secs());

    err_to_resp!(stream.write_all(msg.as_bytes()).await);
    tracing::info!("Wrote to {}", push_test_addr);

    StatusCode::OK.into_response()
}
