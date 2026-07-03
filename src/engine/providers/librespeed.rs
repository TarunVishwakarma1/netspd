//! LibreSpeed provider: server discovery against the public LibreSpeed
//! backend network, with custom and built-in fallbacks.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::engine::models::Server;
use crate::engine::network::client::build_client;
use crate::errors::{EngineError, EngineResult};

use super::Provider;

/// The official public server list.
const SERVER_LIST_URL: &str = "https://librespeed.org/backend-servers/servers.php";

/// Timeout for the server list request; discovery must never hang the UI.
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(6);

/// Download chunk-count query appended when a `dlURL` lacks one.
/// Each chunk is roughly one megabyte on LibreSpeed backends.
const DOWNLOAD_QUERY: &str = "ckSize=100";

/// A provider backed by LibreSpeed-compatible servers.
///
/// Resolution order:
/// 1. custom servers from user configuration, when present;
/// 2. the public LibreSpeed server list;
/// 3. a built-in list of well-known public backends.
pub struct LibreSpeedProvider {
    client: Client,
    custom_servers: Vec<Server>,
}

impl LibreSpeedProvider {
    /// Creates the provider, optionally seeded with custom servers.
    pub fn new(custom_servers: Vec<Server>) -> EngineResult<Self> {
        Ok(Self {
            client: build_client(DISCOVERY_TIMEOUT, 1)?,
            custom_servers,
        })
    }

    /// Fetches and parses the public server list.
    async fn fetch_remote(&self) -> EngineResult<Vec<Server>> {
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

#[async_trait]
impl Provider for LibreSpeedProvider {
    fn name(&self) -> &'static str {
        "LibreSpeed"
    }

    async fn fetch_servers(&self) -> EngineResult<Vec<Server>> {
        if !self.custom_servers.is_empty() {
            return Ok(self.custom_servers.clone());
        }
        match self.fetch_remote().await {
            Ok(servers) if !servers.is_empty() => Ok(servers),
            // Remote discovery is best-effort: fall back to known-good
            // public backends rather than failing the whole application.
            Ok(_) | Err(_) => Ok(builtin_servers()),
        }
    }
}

/// One entry of the LibreSpeed server list JSON.
///
/// Every field is optional because the list is community-maintained;
/// incomplete entries are skipped rather than trusted.
#[derive(Debug, Deserialize)]
struct RawServer {
    name: Option<String>,
    server: Option<String>,
    #[serde(rename = "dlURL")]
    dl_url: Option<String>,
    #[serde(rename = "ulURL")]
    ul_url: Option<String>,
    #[serde(rename = "pingURL")]
    ping_url: Option<String>,
}

/// Parses the LibreSpeed public server list JSON into validated servers.
///
/// Entries missing any required field are dropped. Download URLs get a
/// chunk-count query when the entry does not provide one.
pub fn parse_server_list(json: &str) -> EngineResult<Vec<Server>> {
    let raw: Vec<RawServer> = serde_json::from_str(json)
        .map_err(|err| EngineError::InvalidResponse(format!("malformed server list: {err}")))?;

    let servers = raw
        .into_iter()
        .filter_map(|entry| {
            let name = entry.name?;
            let base = entry.server?;
            let dl = entry.dl_url?;
            let ul = entry.ul_url?;
            let ping = entry.ping_url?;
            if name.trim().is_empty() || base.trim().is_empty() {
                return None;
            }
            Some(Server::from_base(
                name.trim(),
                &base,
                &with_download_query(&dl),
                &ul,
                &ping,
            ))
        })
        .collect();
    Ok(servers)
}

/// Ensures the download path requests a large payload.
fn with_download_query(path: &str) -> String {
    if path.contains("ckSize=") {
        path.to_owned()
    } else if path.contains('?') {
        format!("{path}&{DOWNLOAD_QUERY}")
    } else {
        format!("{path}?{DOWNLOAD_QUERY}")
    }
}

/// Well-known public LibreSpeed backends used when discovery fails.
#[must_use]
pub fn builtin_servers() -> Vec<Server> {
    const BACKENDS: [(&str, &str); 4] = [
        (
            "Frankfurt, Germany (Clouvider)",
            "fra.speedtest.clouvider.net/backend",
        ),
        (
            "London, United Kingdom (Clouvider)",
            "lon.speedtest.clouvider.net/backend",
        ),
        (
            "New York, USA (Clouvider)",
            "nyc.speedtest.clouvider.net/backend",
        ),
        (
            "Los Angeles, USA (Clouvider)",
            "la.speedtest.clouvider.net/backend",
        ),
    ];
    BACKENDS
        .into_iter()
        .map(|(name, base)| {
            Server::from_base(
                name,
                base,
                &with_download_query("garbage.php"),
                "empty.php",
                "empty.php",
            )
        })
        .collect()
}
