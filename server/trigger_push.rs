use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use pusher::err_to_resp;
use reqwest::StatusCode;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;

#[derive(Debug, serde::Deserialize)]
pub struct Message {
    message: String,
}

/// Write message body into socket, so that `push-sender` can forward it as
/// a push message. Note that this could be combined into a single binary..?
pub async fn write_to_socket(
    State(push_test_addr): State<Option<Arc<str>>>,
    Json(msg): Json<Message>,
) -> Response {
    let Some(push_test_addr) = push_test_addr else {
        tracing::info!("Trying to write without socket addr");
        return StatusCode::OK.into_response();
    };
    let mut stream = err_to_resp!(UnixStream::connect(push_test_addr.as_ref()).await);

    err_to_resp!(stream.write_all(msg.message.as_bytes()).await);
    tracing::info!("Wrote to {}", push_test_addr);

    StatusCode::OK.into_response()
}

#[derive(Debug, serde::Serialize)]
pub struct SocketExists {
    exists: bool,
}

/// Check if the socket exists and return it as JSON
pub async fn socket_exists(State(exists): State<bool>) -> Response {
    (StatusCode::OK, Json(SocketExists { exists })).into_response()
}
