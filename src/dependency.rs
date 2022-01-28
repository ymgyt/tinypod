use std::env;

use anyhow::anyhow;
use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct Dependency {
    pub db_pool: Option<PgPool>,
    pub mongo_client: Option<mongodb::Client>,
}

impl Dependency {
    pub async fn new() -> anyhow::Result<Self> {
        let db_pool = connect_db_if_needed().await;
        let mongo_client = connect_mongodb_if_needed().await;

        Ok(Self {
            db_pool,
            mongo_client,
        })
    }
}

async fn connect_db() -> anyhow::Result<PgPool> {
    // use DB_URI if specified
    let uri = if let Ok(uri) = env::var("TINYPOD_DB_URI") {
        uri
    } else {
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

        uri
    };

    tracing::debug!(%uri, "connecting to db");

    Ok(PgPool::connect(&uri).await?)
}

async fn connect_db_if_needed() -> Option<PgPool> {
    let connect = if let Ok(raw) = env::var("TINYPOD_DB_USE") {
        raw.parse::<bool>()
            .unwrap_or_else(|_| panic!("TINYPOD_DB_USE should be 'true' or 'false'. got {raw}"))
    } else {
        false
    };
    if connect {
        Some(connect_db().await.unwrap())
    } else {
        None
    }
}

async fn connect_mongodb() -> anyhow::Result<mongodb::Client> {
    let connection_string = env::var("TINYPOD_MONGODB_CONNECTION_STRING")
        .map_err(|_| anyhow!("environment variable TINYPOD_MONGODB_CONNECTION_STRING required"))?;

    let options = mongodb::options::ClientOptions::parse(&connection_string).await?;

    let client = mongodb::Client::with_options(options)?;

    Ok(client)
}

async fn connect_mongodb_if_needed() -> Option<mongodb::Client> {
    let connect = if let Ok(raw) = env::var("TINYPOD_MONGODB_USE") {
        raw.parse::<bool>().unwrap_or_else(|_| panic!(
            "TINYPOD_MONGODB_USE should be 'true' or 'false. got {raw}"
        ))
    } else {
        false
    };
    if connect {
        Some(connect_mongodb().await.unwrap())
    } else {
        None
    }
}
