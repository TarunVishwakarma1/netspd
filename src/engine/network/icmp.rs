//! ICMP echo probing for real packet loss measurement.
//!
//! HTTP request failures only approximate loss; a burst of ICMP echoes
//! measures it directly. ICMP sockets are a privilege-dependent luxury —
//! unavailable sockets, blocked ICMP or resolution failures all
//! degrade gracefully to `None`, and the caller keeps the HTTP-based
//! estimate.

use std::net::IpAddr;
use std::time::Duration;

use surge_ping::{Client, Config, PingIdentifier, PingSequence, ICMP};
use tokio_util::sync::CancellationToken;

/// Number of echo requests per measurement.
const PACKETS: u16 = 20;
/// Pause between echoes.
const INTERVAL: Duration = Duration::from_millis(50);
/// Per-echo timeout; slower replies count as lost.
const TIMEOUT: Duration = Duration::from_secs(1);
/// Echo payload (timestamps and identifiers live in the ICMP header).
const PAYLOAD: [u8; 16] = [0; 16];

/// Extracts the host from an HTTP(S) URL, e.g.
/// `https://host.example.com/backend/empty.php` → `host.example.com`.
#[must_use]
pub fn host_of_url(url: &str) -> Option<String> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let host_port = rest.split(['/', '?']).next()?;
    let host = host_port
        .rsplit_once(':')
        .map_or(host_port, |(host, port)| {
            // Only treat the suffix as a port when it is numeric; IPv6
            // literals contain colons too.
            if port.chars().all(|ch| ch.is_ascii_digit()) {
                host
            } else {
                host_port
            }
        });
    if host.is_empty() {
        None
    } else {
        Some(host.to_owned())
    }
}

/// Measures packet loss to `host` with a burst of ICMP echoes.
///
/// Returns the loss percentage in `0.0..=100.0`, or `None` when ICMP is
/// unusable: no socket permission, resolution failure, cancellation, or
/// 100% loss (which almost always means a firewall dropping ICMP rather
/// than a dead link — the HTTP estimate is more honest then).
pub async fn measure_loss(host: &str, cancel: &CancellationToken) -> Option<f64> {
    let address = resolve(host).await?;
    let kind = match address {
        IpAddr::V4(_) => ICMP::V4,
        IpAddr::V6(_) => ICMP::V6,
    };
    let client = Client::new(&Config::builder().kind(kind).build()).ok()?;
    let mut pinger = client
        .pinger(address, PingIdentifier(std::process::id() as u16))
        .await;
    pinger.timeout(TIMEOUT);

    let mut lost: u32 = 0;
    for sequence in 0..PACKETS {
        let outcome = tokio::select! {
            () = cancel.cancelled() => return None,
            outcome = pinger.ping(PingSequence(sequence), &PAYLOAD) => outcome,
        };
        if outcome.is_err() {
            lost += 1;
        }
        if sequence + 1 < PACKETS {
            tokio::select! {
                () = cancel.cancelled() => return None,
                () = tokio::time::sleep(INTERVAL) => {}
            }
        }
    }

    if lost == u32::from(PACKETS) {
        return None;
    }
    Some(f64::from(lost) / f64::from(PACKETS) * 100.0)
}

/// Resolves a host name to one IP address, preferring IPv4.
async fn resolve(host: &str) -> Option<IpAddr> {
    let addresses: Vec<IpAddr> = tokio::net::lookup_host((host, 0))
        .await
        .ok()?
        .map(|socket| socket.ip())
        .collect();
    addresses
        .iter()
        .find(|address| address.is_ipv4())
        .or_else(|| addresses.first())
        .copied()
}
