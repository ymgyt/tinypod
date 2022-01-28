use std::{
    borrow::Cow,
    future::Future,
    net::{SocketAddr, TcpListener},
};

use axum::{extract::Extension, response::IntoResponse, routing::get, AddExtensionLayer, Router};
use tower_http::trace::TraceLayer;
use tracing::info;
use itertools::Itertools;

use crate::dependency::Dependency;

async fn healthcheck() -> &'static str {
    "OK"
}

pub async fn run(
    addr: impl Into<SocketAddr>,
    shutdown: impl Future<Output = ()>,
    dependency: Dependency,
) -> Result<(), anyhow::Error> {
    let addr = addr.into();
    let listener = std::net::TcpListener::bind(addr)?;

    info!(?addr, "listening");

    run_with_listener(listener, shutdown,dependency).await
}

async fn ping(Extension(dep): Extension<Dependency>) -> impl IntoResponse {
    use sqlx::Connection;
    if let Some(pool) = dep.db_pool {
        match pool.acquire().await.unwrap().ping().await {
            Ok(_) => Cow::Borrowed("ping success"),
            Err(err) => Cow::Owned(format!("ping failed: {:?}", err)),
        }
    } else {
        Cow::Borrowed("no db connection")
    }
}

async fn list_mongodb_databases(Extension(dep): Extension<Dependency>) -> impl IntoResponse {
    if let Some(client) = dep.mongo_client {
         Cow::Owned(client.list_database_names(None,None).await.unwrap().into_iter().join("\n"))
    } else {
       Cow::Borrowed("no mongodb connection")
    }
}

pub async fn run_with_listener(
    listener: TcpListener,
    shutdown: impl Future<Output = ()>,
    dependency: Dependency,
) -> Result<(), anyhow::Error> {
    let app = Router::new()
        .route("/db/ping", get(ping))
        .route("mongodb/databases", get(list_mongodb_databases))
        .route("/healthcheck", get(healthcheck))
        .layer(
            tower::ServiceBuilder::new()
                .layer(TraceLayer::new_for_http().on_request(
                    |request: &axum::http::Request<_>, _: &tracing::Span| {
                        tracing::info!(?request);
                    },
                ))
                .layer(AddExtensionLayer::new(dependency)),
        );

    axum::Server::from_tcp(listener)?
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown)
        .await
        .map_err(anyhow::Error::from)
}
