//! Live view types for each test phase, updated by engine events.

use std::time::Duration;

use crate::engine::metrics::Sampler;
use crate::engine::models::{LatencyStats, TransferStats};

/// Number of speed samples kept for sparklines.
pub(super) const HISTORY_CAPACITY: usize = 120;

/// Live view of the ping phase.
#[derive(Debug, Clone)]
pub struct PingView {
    /// Most recent sample, in milliseconds.
    pub last_ms: Option<f64>,
    /// Number of samples completed so far.
    pub samples_done: u32,
    /// Final statistics, once the phase completes.
    pub stats: Option<LatencyStats>,
    /// Recent samples for the sparkline.
    pub history: Sampler,
}

impl PingView {
    pub(super) fn new() -> Self {
        Self {
            last_ms: None,
            samples_done: 0,
            stats: None,
            history: Sampler::new(HISTORY_CAPACITY),
        }
    }

    pub(super) fn reset(&mut self) {
        self.last_ms = None;
        self.samples_done = 0;
        self.stats = None;
        self.history.clear();
    }
}

/// Live view of a download or upload phase.
#[derive(Debug, Clone)]
pub struct TransferView {
    /// Bytes transferred so far.
    pub bytes: u64,
    /// Smoothed instantaneous speed, in bits per second.
    pub current_bps: f64,
    /// Average speed, in bits per second.
    pub average_bps: f64,
    /// Peak smoothed speed, in bits per second.
    pub peak_bps: f64,
    /// Estimated time remaining in this phase.
    pub eta: Duration,
    /// Phase completion in `0.0..=1.0`.
    pub ratio: f64,
    /// Recent speed samples for the sparkline.
    pub history: Sampler,
    /// Final statistics, once the phase completes.
    pub stats: Option<TransferStats>,
}

impl TransferView {
    pub(super) fn new() -> Self {
        Self {
            bytes: 0,
            current_bps: 0.0,
            average_bps: 0.0,
            peak_bps: 0.0,
            eta: Duration::ZERO,
            ratio: 0.0,
            history: Sampler::new(HISTORY_CAPACITY),
            stats: None,
        }
    }

    pub(super) fn reset(&mut self) {
        self.bytes = 0;
        self.current_bps = 0.0;
        self.average_bps = 0.0;
        self.peak_bps = 0.0;
        self.eta = Duration::ZERO;
        self.ratio = 0.0;
        self.history.clear();
        self.stats = None;
    }
}
