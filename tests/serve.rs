//! Integration tests for the built-in speed test server.
//!
//! A real listener is spawned on an ephemeral port and exercised with a
//! real HTTP client — fully offline.

use netspd::app::serve;

async fn start_server() -> Result<u16, Box<dyn std::error::Error>> {
    // Find a free port, release it, then bind netspd's server there.
    let probe = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = probe.local_addr()?.port();
    drop(probe);
    tokio::spawn(async move {
        let _ = serve::run("127.0.0.1", port).await;
    });
    // Wait for the listener to come up.
    for _ in 0..50 {
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            return Ok(port);
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    Err("server did not start".into())
}

#[tokio::test]
async fn serves_ping_download_upload() -> Result<(), Box<dyn std::error::Error>> {
    let port = start_server().await?;
    let base = format!("http://127.0.0.1:{port}");
    let client = reqwest::Client::new();

    // Ping, both native and LibreSpeed paths.
    for path in ["/ping", "/empty.php?r=1"] {
        let response = client.get(format!("{base}{path}")).send().await?;
        assert!(response.status().is_success(), "GET {path}");
    }

    // Download honors ?bytes=N exactly.
    let body = client
        .get(format!("{base}/download?bytes=100000"))
        .send()
        .await?
        .bytes()
        .await?;
    assert_eq!(body.len(), 100_000);
    // Payload is incompressible, not zeros.
    assert!(body.iter().any(|&b| b != 0));

    // LibreSpeed ckSize is megabytes.
    let body = client
        .get(format!("{base}/garbage.php?ckSize=1"))
        .send()
        .await?
        .bytes()
        .await?;
    assert_eq!(body.len(), 1024 * 1024);

    // Upload drains arbitrary bodies.
    let response = client
        .post(format!("{base}/upload"))
        .body(vec![7u8; 262_144])
        .send()
        .await?;
    assert!(response.status().is_success());

    // Streaming (chunked) uploads — what netspd's own client sends —
    // are drained correctly and keep-alive survives.
    let chunks: Vec<Result<Vec<u8>, std::io::Error>> =
        vec![Ok(vec![1u8; 65536]), Ok(vec![2u8; 65536])];
    let stream = futures_util::stream::iter(chunks);
    let response = client
        .post(format!("{base}/upload"))
        .body(reqwest::Body::wrap_stream(stream))
        .send()
        .await?;
    assert!(response.status().is_success());
    let response = client.get(format!("{base}/ping")).send().await?;
    assert!(response.status().is_success(), "keep-alive after chunked");

    // Unknown paths 404 without closing the connection.
    let response = client.get(format!("{base}/nope")).send().await?;
    assert_eq!(response.status().as_u16(), 404);
    let response = client.get(format!("{base}/ping")).send().await?;
    assert!(response.status().is_success());
    Ok(())
}
