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

/// Bufferbloat grades, from best to worst, following the thresholds
/// popularized by Waveform's bufferbloat test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BufferbloatGrade {
    /// Latency rises less than 5 ms under load.
    APlus,
    /// Less than 30 ms of added latency.
    A,
    /// Less than 60 ms.
    B,
    /// Less than 200 ms.
    C,
    /// Less than 400 ms.
    D,
    /// 400 ms or more — the connection collapses under load.
    F,
}

impl BufferbloatGrade {
    /// Grades the worst latency increase observed under load.
    #[must_use]
    pub fn from_increase_ms(increase_ms: f64) -> Self {
        match increase_ms {
            x if x < 5.0 => Self::APlus,
            x if x < 30.0 => Self::A,
            x if x < 60.0 => Self::B,
            x if x < 200.0 => Self::C,
            x if x < 400.0 => Self::D,
            _ => Self::F,
        }
    }

    /// Display label, e.g. `A+`.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::APlus => "A+",
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::F => "F",
        }
    }
}

/// Latency measured while the link is saturated, versus idle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bufferbloat {
    /// Median latency during the download phase, in milliseconds.
    pub download_ms: f64,
    /// Median latency during the upload phase, in milliseconds.
    pub upload_ms: f64,
    /// Idle latency the increases are measured against, in milliseconds.
    pub idle_ms: f64,
    /// Overall grade from the worst direction.
    pub grade: BufferbloatGrade,
}

impl Bufferbloat {
    /// Builds the stats and grade from idle and loaded medians.
    #[must_use]
    pub fn new(idle_ms: f64, download_ms: f64, upload_ms: f64) -> Self {
        let worst = (download_ms - idle_ms).max(upload_ms - idle_ms).max(0.0);
        Self {
            download_ms,
            upload_ms,
            idle_ms,
            grade: BufferbloatGrade::from_increase_ms(worst),
        }
    }
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
    /// Latency under load, when the sampler collected enough data.
    pub bufferbloat: Option<Bufferbloat>,
}
