//! Shared test fixtures for integration tests.
//!
//! Every test file that needs a `TestReport` should use `ReportBuilder`
//! rather than constructing the struct inline.  When new fields are added
//! only this file needs updating.

use std::time::Duration;

use netspd::engine::models::{Bufferbloat, IpVersion, LatencyStats, TestReport, TransferStats};

/// Default values used when a field is not overridden by the builder.
const DEFAULT_SERVER: &str = "Tokyo, Japan (A573)";
const DEFAULT_DOWN_MBPS: f64 = 104.9;
const DEFAULT_UP_MBPS: f64 = 36.3;
const DEFAULT_PING_MS: f64 = 141.2;
const DEFAULT_JITTER_MS: f64 = 1.0;
const DEFAULT_LOSS_PCT: f64 = 0.0;

fn transfer(mbps: f64) -> TransferStats {
    TransferStats {
        bytes: (mbps * 1_000_000.0 * 10.0) as u64,
        duration: Duration::from_secs(10),
        average_bps: mbps * 1_000_000.0,
        peak_bps: mbps * 1_000_000.0 * 2.0,
    }
}

fn latency(avg_ms: f64, jitter_ms: f64, loss_pct: f64) -> LatencyStats {
    LatencyStats {
        average_ms: avg_ms,
        jitter_ms,
        min_ms: avg_ms * 0.9,
        max_ms: avg_ms * 1.1,
        samples: 10,
        packet_loss_pct: loss_pct,
    }
}

/// Fluent builder for `TestReport` in integration tests.
pub struct ReportBuilder {
    server_name: String,
    down_mbps: f64,
    up_mbps: f64,
    ping_ms: f64,
    jitter_ms: f64,
    loss_pct: f64,
    bufferbloat: Option<Bufferbloat>,
    ip_version: Option<IpVersion>,
}

impl Default for ReportBuilder {
    fn default() -> Self {
        Self {
            server_name: DEFAULT_SERVER.to_owned(),
            down_mbps: DEFAULT_DOWN_MBPS,
            up_mbps: DEFAULT_UP_MBPS,
            ping_ms: DEFAULT_PING_MS,
            jitter_ms: DEFAULT_JITTER_MS,
            loss_pct: DEFAULT_LOSS_PCT,
            bufferbloat: None,
            ip_version: None,
        }
    }
}

#[allow(dead_code)]
impl ReportBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn server(mut self, name: &str) -> Self {
        self.server_name = name.to_owned();
        self
    }

    pub fn download(mut self, mbps: f64) -> Self {
        self.down_mbps = mbps;
        self
    }

    pub fn upload(mut self, mbps: f64) -> Self {
        self.up_mbps = mbps;
        self
    }

    pub fn ping(mut self, avg_ms: f64) -> Self {
        self.ping_ms = avg_ms;
        self
    }

    pub fn jitter(mut self, ms: f64) -> Self {
        self.jitter_ms = ms;
        self
    }

    pub fn loss(mut self, pct: f64) -> Self {
        self.loss_pct = pct;
        self
    }

    pub fn bufferbloat(mut self, idle_ms: f64, down_ms: f64, up_ms: f64) -> Self {
        self.bufferbloat = Some(Bufferbloat::new(idle_ms, down_ms, up_ms));
        self
    }

    pub fn ip_version(mut self, version: IpVersion) -> Self {
        self.ip_version = Some(version);
        self
    }

    pub fn build(self) -> TestReport {
        TestReport {
            server_name: self.server_name,
            latency: latency(self.ping_ms, self.jitter_ms, self.loss_pct),
            download: transfer(self.down_mbps),
            upload: transfer(self.up_mbps),
            bufferbloat: self.bufferbloat,
            ip_version: self.ip_version,
        }
    }
}
