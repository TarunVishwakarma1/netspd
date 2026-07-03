//! Client IP and ISP discovery via the server's `getIP` endpoint.

use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;

/// Timeout for the info request; this is cosmetic data and must never
/// delay the test.
const INFO_TIMEOUT: Duration = Duration::from_secs(4);

/// Longest string shown in the UI.
const MAX_LEN: usize = 64;

/// The JSON shape returned by LibreSpeed's `getIP.php?isp=true`.
#[derive(Debug, Deserialize)]
struct IpInfo {
    #[serde(rename = "processedString")]
    processed: Option<String>,
}

/// Derives the `getIP` URL from a server's ping URL (both live in the
/// same backend directory on LibreSpeed-compatible servers).
#[must_use]
pub fn info_url(ping_url: &str) -> String {
    let base = ping_url
        .split('?')
        .next()
        .unwrap_or(ping_url)
        .rsplit_once('/')
        .map_or("", |(base, _)| base);
    format!("{base}/getIP.php?isp=true")
}

/// Fetches the client's public IP and ISP as a display string.
///
/// Best-effort: any failure (timeout, HTTP error, unexpected body)
/// yields `None` and the UI simply omits the line.
pub async fn fetch_client_info(client: &Client, ping_url: &str) -> Option<String> {
    let url = info_url(ping_url);
    let response = tokio::time::timeout(INFO_TIMEOUT, client.get(url).send())
        .await
        .ok()?
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let body = tokio::time::timeout(INFO_TIMEOUT, response.text())
        .await
        .ok()?
        .ok()?;

    let text = serde_json::from_str::<IpInfo>(&body)
        .ok()
        .and_then(|info| info.processed)
        .unwrap_or(body);
    let cleaned = text.trim();
    if cleaned.is_empty() || cleaned.len() > MAX_LEN * 4 {
        return None;
    }
    Some(cleaned.chars().take(MAX_LEN).collect())
}
