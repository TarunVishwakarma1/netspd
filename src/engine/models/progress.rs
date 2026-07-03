//! Live progress snapshots emitted during transfer phases.

use std::time::Duration;

use super::TestPhase;

/// A point-in-time snapshot of an in-flight transfer.
///
/// Emitted several times per second while a download or upload runs.
/// Speeds are expressed in bits per second.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransferProgress {
    /// The phase this snapshot belongs to.
    pub phase: TestPhase,
    /// Total bytes transferred so far.
    pub bytes_transferred: u64,
    /// Time elapsed since the phase started.
    pub elapsed: Duration,
    /// EMA-smoothed instantaneous speed, in bits per second.
    pub current_bps: f64,
    /// Average speed across the whole phase, in bits per second.
    pub average_bps: f64,
    /// Highest smoothed speed observed so far, in bits per second.
    pub peak_bps: f64,
    /// Estimated time until the phase completes.
    pub eta: Duration,
    /// Completion ratio in `0.0..=1.0`, based on the phase duration.
    pub ratio: f64,
}
