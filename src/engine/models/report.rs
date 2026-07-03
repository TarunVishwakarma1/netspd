//! Final measurement results.

use std::time::Duration;

/// Aggregated latency statistics from the ping phase.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LatencyStats {
    /// Trimmed mean latency, in milliseconds.
    pub average_ms: f64,
    /// Mean absolute difference between successive samples, in milliseconds.
    pub jitter_ms: f64,
    /// Fastest sample, in milliseconds.
    pub min_ms: f64,
    /// Slowest sample, in milliseconds.
    pub max_ms: f64,
    /// Number of successful samples.
    pub samples: usize,
    /// Percentage of samples that failed, in `0.0..=100.0`.
    ///
    /// HTTP-based providers approximate packet loss with request failures;
    /// ICMP-based measurement can slot in here later without API changes.
    pub packet_loss_pct: f64,
}

/// Aggregated statistics from a completed transfer phase.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransferStats {
    /// Total bytes transferred.
    pub bytes: u64,
    /// Wall-clock duration of the phase.
    pub duration: Duration,
    /// Average speed across the phase, in bits per second.
    pub average_bps: f64,
    /// Highest smoothed speed observed, in bits per second.
    pub peak_bps: f64,
}

/// The complete result of a speed test run.
#[derive(Debug, Clone, PartialEq)]
pub struct TestReport {
    /// Name of the server the test ran against.
    pub server_name: String,
    /// Latency and jitter results.
    pub latency: LatencyStats,
    /// Download phase results.
    pub download: TransferStats,
    /// Upload phase results.
    pub upload: TransferStats,
}
