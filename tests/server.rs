#[tokio::test]
async fn server_test() -> Result<(), anyhow::Error> {
    let addr = "localhost:8002";
    let listener = std::net::TcpListener::bind(addr).unwrap();

    let (tx, rx) = tokio::sync::oneshot::channel();

    let server_handle = tokio::spawn(tinypod::server::run_with_listener(listener, async {
        rx.await.unwrap();
    }));

    assert_eq!(
        reqwest::get("http://localhost:8002/healthcheck")
            .await?
            .text()
            .await?,
        String::from("OK")
    );

    tx.send(()).unwrap();
    server_handle.await??;

    Ok(())
}
