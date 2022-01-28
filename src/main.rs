use tinypod::server;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(true)
        .init();

    let signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for event");
        tracing::info!("receive sigint");
    };

    let addr = ([0, 0, 0, 0], 8001);
    let dep = tinypod::Dependency::new().await?;

    server::run(addr, signal, dep).await
}
