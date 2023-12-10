use crate::vapid;
use axum::response::{Redirect, Response};
use axum::routing::{get, post};
use axum::Server;
use pusher::db::get_pool;
use pusher::err::{PusherError, Res};
use pusher::subscription::{subscribe, unsubscribe};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::Level;

fn log_status<B, D, S>(response: &Response<B>, _latency: D, _span: &S) {
    let stat = response.status();
    if stat.is_client_error() || stat.is_server_error() {
        tracing::error!("{}", stat)
    }
}

#[tokio::main]
pub async fn run(
    pubkey: vapid::PublicKey,
    addr: SocketAddr,
    encryption_key: [u8; 16],
    db_url: &str,
) -> Res<()> {
    tracing_subscriber::fmt::fmt()
        .with_max_level(Level::INFO)
        .init();

    let pool = get_pool(db_url, false).await?;

    let trace = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(log_status);

    let app = axum::Router::new()
        .nest("/vapid", vapid::router())
        .with_state(pubkey)
        .route("/subscribe", post(subscribe))
        .route("/unsubscribe", post(unsubscribe))
        .with_state((pool, encryption_key))
        .route("/", get(Redirect::to("/index.html")))
        .fallback_service(ServeDir::new("assets"))
        .layer(trace);

    tracing::info!("listening on {}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| PusherError::from(e.to_string()))
}
