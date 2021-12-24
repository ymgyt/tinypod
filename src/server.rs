use std::{
    future::Future,
    net::{SocketAddr, TcpListener},
};

use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;
use tracing::info;

async fn healthcheck() -> &'static str {
    "OK"
}

pub async fn run(
    addr: impl Into<SocketAddr>,
    shutdown: impl Future<Output = ()>,
) -> Result<(), anyhow::Error> {
    let addr = addr.into();
    let listener = std::net::TcpListener::bind(addr)?;

    info!(?addr, "listening");

    run_with_listener(listener, shutdown).await
}

pub async fn run_with_listener(
    listener: TcpListener,
    shutdown: impl Future<Output = ()>,
) -> Result<(), anyhow::Error> {
    let app = Router::new().route("/healthcheck", get(healthcheck)).layer(
        tower::ServiceBuilder::new().layer(TraceLayer::new_for_http().on_request(
            |request: &axum::http::Request<_>, _: &tracing::Span| {
                tracing::info!(?request);
            },
        )),
    );

    axum::Server::from_tcp(listener)?
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown)
        .await
        .map_err(anyhow::Error::from)
}
