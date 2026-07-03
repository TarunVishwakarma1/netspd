//! Tests for server health probing.

use std::time::Duration;

use netspd::engine::models::Server;
use netspd::engine::network::client::build_client;
use netspd::engine::network::health::{filter_reachable, probe};

fn unreachable_server(name: &str) -> Server {
    // Port 1 on loopback: connection refused immediately, no network
    // dependency in tests.
    Server::from_base(
        name,
        "http://127.0.0.1:1/",
        "garbage.php?ckSize=100",
        "empty.php",
        "empty.php",
    )
}

#[tokio::test]
async fn probe_returns_none_for_unreachable_server() -> Result<(), Box<dyn std::error::Error>> {
    let client = build_client(Duration::from_secs(1), 1)?;
    let server = unreachable_server("Dead");
    assert!(probe(&client, &server).await.is_none());
    Ok(())
}

#[tokio::test]
async fn filter_reachable_drops_dead_servers() -> Result<(), Box<dyn std::error::Error>> {
    let client = build_client(Duration::from_secs(1), 1)?;
    let servers = vec![unreachable_server("Dead A"), unreachable_server("Dead B")];
    let reachable = filter_reachable(&client, servers).await;
    assert!(reachable.is_empty());
    Ok(())
}
