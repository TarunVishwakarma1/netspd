//! Fast.com (Netflix) provider.
//!
//! Netflix exposes speed test targets on its CDN through a public API;
//! the token below is the one embedded in fast.com's own web client and
//! shared by every open-source Fast.com tool. Targets serve arbitrary
//! byte ranges via GET and accept POST bodies, mapping directly onto
//! netspd's download and upload endpoints.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::engine::models::{Endpoints, Server};
use crate::engine::network::client::build_client;
use crate::errors::{EngineError, EngineResult};

use super::Provider;

/// The public API token embedded in fast.com's web client.
const API_TOKEN: &str = "YXNkZmFzZGxmbnNkYWZoYXNkZmhrYWxm";

/// Number of CDN targets requested.
const TARGET_COUNT: u8 = 5;

/// Timeout for target discovery.
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(6);

/// Bytes requested per download range (~25 MB); workers loop requests.
const DOWNLOAD_RANGE_BYTES: u64 = 26_214_400;

/// A provider backed by Netflix's Fast.com CDN targets.
pub struct FastProvider {
    client: Client,
}

impl FastProvider {
    /// Creates the provider.
    pub fn new() -> EngineResult<Self> {
        Ok(Self {
            client: build_client(DISCOVERY_TIMEOUT, 1)?,
        })
    }
}

#[async_trait]
impl Provider for FastProvider {
    fn name(&self) -> &'static str {
        "Fast.com"
    }

    async fn fetch_servers(&self) -> EngineResult<Vec<Server>> {
        let url = format!(
            "https://api.fast.com/netflix/speedtest/v2?https=true&token={API_TOKEN}&urlCount={TARGET_COUNT}"
        );
        let response = tokio::time::timeout(DISCOVERY_TIMEOUT, self.client.get(url).send())
            .await
            .map_err(|_| EngineError::InvalidResponse("target request timed out".to_owned()))??;
        if !response.status().is_success() {
            return Err(EngineError::InvalidResponse(format!(
                "target API returned HTTP {}",
                response.status()
            )));
        }
        let body = response.text().await?;
        parse_targets(&body)
    }
}

/// The relevant slice of the Fast.com API response.
#[derive(Debug, Deserialize)]
struct RawResponse {
    targets: Vec<RawTarget>,
}

#[derive(Debug, Deserialize)]
struct RawTarget {
    url: Option<String>,
    location: Option<RawLocation>,
}

#[derive(Debug, Deserialize)]
struct RawLocation {
    city: Option<String>,
    country: Option<String>,
}

/// Parses the Fast.com target list into validated servers.
pub fn parse_targets(json: &str) -> EngineResult<Vec<Server>> {
    let raw: RawResponse = serde_json::from_str(json)
        .map_err(|err| EngineError::InvalidResponse(format!("malformed target list: {err}")))?;

    let servers = raw
        .targets
        .into_iter()
        .filter_map(|target| {
            let url = target.url?;
            if !url.starts_with("https://") {
                return None;
            }
            let host = url
                .strip_prefix("https://")
                .and_then(|rest| rest.split('/').next())
                .unwrap_or_default()
                .to_owned();
            if host.is_empty() {
                return None;
            }
            let name = match target.location {
                Some(RawLocation {
                    city: Some(city),
                    country: Some(country),
                }) => format!("Fast.com — {city}, {country}"),
                _ => format!("Fast.com — {host}"),
            };
            Some(Server {
                name,
                description: host,
                endpoints: Endpoints {
                    // A zero-byte range answers instantly: ideal for
                    // latency samples and health probes.
                    ping: with_range(&url, 0),
                    download: with_range(&url, DOWNLOAD_RANGE_BYTES),
                    upload: with_range(&url, DOWNLOAD_RANGE_BYTES),
                },
                probe_ms: None,
            })
        })
        .collect();
    Ok(servers)
}

/// Inserts a `/range/0-N` segment into a target URL, preserving its
/// signed query string.
fn with_range(url: &str, bytes: u64) -> String {
    match url.split_once('?') {
        Some((path, query)) => format!("{path}/range/0-{bytes}?{query}"),
        None => format!("{url}/range/0-{bytes}"),
    }
}
