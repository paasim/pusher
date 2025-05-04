use crate::trigger_push::write_to_socket;
use crate::{vapid, Config};
use axum::response::{Redirect, Response};
use axum::routing::{get, post};
use pusher::db::get_pool;
use pusher::err::Result;
use pusher::subscription::{subscribe, unsubscribe};
use tokio::net::TcpListener;
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
pub async fn run(conf: Config) -> Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_max_level(Level::INFO)
        .init();

    let pool = get_pool(&conf.db_path, false)?;

    let trace = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(log_status);
    let tmp_path = conf.push_test_addr.map(|s| s.into());

    let app = axum::Router::new()
        .nest("/vapid", vapid::router())
        .with_state(conf.pubkey)
        .route("/subscribe", post(subscribe))
        .route("/unsubscribe", post(unsubscribe))
        .with_state((pool, conf.encryption_key))
        .route("/test-push", post(write_to_socket))
        .with_state(tmp_path)
        .route("/", get(Redirect::to("/index.html")))
        .fallback_service(ServeDir::new("assets"))
        .layer(trace);

    tracing::info!("listening on {}", conf.listen_addr);

    let listener = TcpListener::bind(conf.listen_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
