//! Ookla (speedtest.net) provider.
//!
//! Uses Ookla's public server directory and the servers' long-standing
//! HTTP endpoints (`latency.txt`, `randomNxN.jpg`, `upload.php`) — the
//! same protocol classic `speedtest-cli` speaks. Servers are addressed
//! through their `host` field (`*.prod.hosts.ooklaserver.net`), which
//! serves HTTPS directly and skips the redirect every legacy URL answers
//! with.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::engine::models::{Endpoints, Server};
use crate::engine::network::client::build_client;
use crate::errors::{EngineError, EngineResult};

use super::Provider;

/// Ookla's public server directory, nearest servers first.
const SERVER_LIST_URL: &str = "https://www.speedtest.net/api/js/servers?engine=js&limit=20";

/// Timeout for server discovery.
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(6);

/// Download asset: ~63 MB of incompressible JPEG per request.
const DOWNLOAD_ASSET: &str = "random4000x4000.jpg";

/// A provider backed by Ookla's speedtest.net server network.
pub struct OoklaProvider {
    client: Client,
}

impl OoklaProvider {
    /// Creates the provider.
    pub fn new() -> EngineResult<Self> {
        Ok(Self {
            client: build_client(DISCOVERY_TIMEOUT, 1)?,
        })
    }
}

#[async_trait]
impl Provider for OoklaProvider {
    fn name(&self) -> &'static str {
        "Ookla"
    }

    async fn fetch_servers(&self) -> EngineResult<Vec<Server>> {
        let response =
            tokio::time::timeout(DISCOVERY_TIMEOUT, self.client.get(SERVER_LIST_URL).send())
                .await
                .map_err(|_| {
                    EngineError::InvalidResponse("server list request timed out".to_owned())
                })??;
        if !response.status().is_success() {
            return Err(EngineError::InvalidResponse(format!(
                "server list returned HTTP {}",
                response.status()
            )));
        }
        let body = response.text().await?;
        parse_server_list(&body)
    }
}

/// One entry of the speedtest.net server directory.
#[derive(Debug, Deserialize)]
struct RawServer {
    host: Option<String>,
    name: Option<String>,
    sponsor: Option<String>,
}

/// Parses the speedtest.net server directory into validated servers.
///
/// Entries without a usable host are dropped rather than trusted.
pub fn parse_server_list(json: &str) -> EngineResult<Vec<Server>> {
    let raw: Vec<RawServer> = serde_json::from_str(json)
        .map_err(|err| EngineError::InvalidResponse(format!("malformed server list: {err}")))?;

    let servers = raw
        .into_iter()
        .filter_map(|entry| {
            let host = entry.host?;
            let city = entry.name?;
            if host.trim().is_empty() || city.trim().is_empty() {
                return None;
            }
            let sponsor = entry.sponsor.unwrap_or_default();
            let name = if sponsor.trim().is_empty() {
                city.trim().to_owned()
            } else {
                format!("{} ({})", city.trim(), sponsor.trim())
            };
            Some(server_for_host(&name, host.trim()))
        })
        .collect();
    Ok(servers)
}

/// Builds a server from an Ookla host (`host.example.net:8080`).
fn server_for_host(name: &str, host: &str) -> Server {
    let base = format!("https://{host}/speedtest/");
    Server {
        name: name.to_owned(),
        description: host.split(':').next().unwrap_or(host).to_owned(),
        endpoints: Endpoints {
            ping: format!("{base}latency.txt"),
            download: format!("{base}{DOWNLOAD_ASSET}"),
            upload: format!("{base}upload.php"),
        },
        probe_ms: None,
    }
}
