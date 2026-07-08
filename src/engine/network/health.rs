//! Server health probing.
//!
//! Public server lists are community-maintained and routinely contain
//! dead, moved or misconfigured entries (expired or mismatched TLS
//! certificates, 404 backends). Probing every server once at discovery
//! keeps them out of the UI entirely and lets the nearest healthy server
//! be selected automatically.

use std::time::{Duration, Instant};

use futures_util::StreamExt;
use reqwest::Client;

use crate::engine::models::Server;

/// Per-server probe timeout: generous enough for distant servers, short
/// enough that a list full of dead entries cannot stall startup.
const PROBE_TIMEOUT: Duration = Duration::from_secs(3);

/// How many probes run in parallel.
const PROBE_CONCURRENCY: usize = 16;

/// Sends one request to a server's ping endpoint and measures the
/// round-trip, returning `None` for any failure (connect, TLS, timeout,
/// error status).
pub async fn probe(client: &Client, server: &Server) -> Option<Duration> {
    let started = Instant::now();
    let response = tokio::time::timeout(
        PROBE_TIMEOUT,
        client
            .get(&server.endpoints.ping)
            .header(reqwest::header::CACHE_CONTROL, "no-cache")
            .send(),
    )
    .await
    .ok()?
    .ok()?;
    response.status().is_success().then(|| started.elapsed())
}

/// Probes all servers concurrently, keeping only the reachable ones,
/// sorted by measured latency (nearest first).
pub async fn filter_reachable(client: &Client, servers: Vec<Server>) -> Vec<Server> {
    let mut healthy: Vec<(Duration, Server)> = futures_util::stream::iter(servers)
        .map(|server| async move {
            let latency = probe(client, &server).await;
            latency.map(|latency| (latency, server))
        })
        .buffer_unordered(PROBE_CONCURRENCY)
        .filter_map(std::future::ready)
        .collect()
        .await;
    healthy.sort_by_key(|(latency, _)| *latency);
    healthy
        .into_iter()
        .map(|(duration, mut server)| {
            server.probe_ms = Some(duration.as_secs_f64() * 1000.0);
            server
        })
        .collect()
}
