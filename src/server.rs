use std::{
    borrow::Cow,
    env,
    future::Future,
    net::{SocketAddr, TcpListener},
};

use anyhow::anyhow;
use axum::{extract::Extension, response::IntoResponse, routing::get, AddExtensionLayer, Router};
use sqlx::postgres::PgPool;
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

async fn connect_db() -> anyhow::Result<PgPool> {
    let user = env::var("TINYPOD_DB_USER")
        .map_err(|_| anyhow!("environment variable TINYPOD_DB_USER required"))?;
    let pass = env::var("TINYPOD_DB_PASS")
        .map_err(|_| anyhow!("environment variable TINYPOD_DB_PASS required"))?;
    let host = env::var("TINYPOD_DB_HOST")
        .map_err(|_| anyhow!("environment variable TINYPOD_DB_HOST required"))?;
    let port = env::var("TINYPOD_DB_PORT")
        .map_err(|_| anyhow!("environment variable TINYPOD_DB_PORT required"))?;
    let name = env::var("TINYPOD_DB_NAME")
        .map_err(|_| anyhow!("environment variable TINYPOD_DB_NAME required"))?;

    let uri = format!(
        "postgresql://{user}:{pass}@{host}:{port}/{name}",
        user = user,
        pass = pass,
        host = host,
        port = port,
        name = name,
    );

    tracing::debug!(%uri, "connecting to db");

    Ok(PgPool::connect(&uri).await?)
}

async fn connect_db_if_needed() -> Option<PgPool> {
    let connect = if let Ok(raw) = env::var("TINYPOD_DB_USE") {
        raw.parse::<bool>()
            .expect("TINYPOD_DB_USE should be 'true' or 'false'")
    } else {
        false
    };
    if connect {
        Some(connect_db().await.unwrap())
    } else {
        None
    }
}

async fn ping(Extension(maybe_pool): Extension<Option<PgPool>>) -> impl IntoResponse {
    use sqlx::Connection;
    if let Some(pool) = maybe_pool {
        match pool.acquire().await.unwrap().ping().await {
            Ok(_) => Cow::Borrowed("ping success"),
            Err(err) => Cow::Owned(format!("ping failed: {:?}", err)),
        }
    } else {
        Cow::Borrowed("no db connection")
    }
}

pub async fn run_with_listener(
    listener: TcpListener,
    shutdown: impl Future<Output = ()>,
) -> Result<(), anyhow::Error> {
    let app = Router::new()
        .route("/db/ping", get(ping))
        .route("/healthcheck", get(healthcheck))
        .layer(
            tower::ServiceBuilder::new()
                .layer(TraceLayer::new_for_http().on_request(
                    |request: &axum::http::Request<_>, _: &tracing::Span| {
                        tracing::info!(?request);
                    },
                ))
                .layer(AddExtensionLayer::new(connect_db_if_needed().await)),
        );

    axum::Server::from_tcp(listener)?
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown)
        .await
        .map_err(anyhow::Error::from)
}
