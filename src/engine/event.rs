//! Strongly-typed events emitted by the engine while a test runs.

use tokio::sync::mpsc;

use crate::errors::{EngineError, EngineResult};

use super::models::{LatencyStats, TestPhase, TestReport, TransferProgress, TransferStats};

/// Events streamed from the engine to any consumer (TUI, CLI, API).
///
/// Consumers never need string matching or magic numbers: every event is a
/// dedicated variant carrying typed payloads.
#[derive(Debug, Clone)]
pub enum EngineEvent {
    /// A new test phase began.
    PhaseStarted {
        /// The phase that just started.
        phase: TestPhase,
    },
    /// A single latency sample completed.
    PingSample {
        /// One-based sample index.
        sequence: u32,
        /// Measured round-trip time, in milliseconds.
        latency_ms: f64,
    },
    /// The ping phase finished with aggregated statistics.
    PingFinished {
        /// Final latency statistics.
        stats: LatencyStats,
    },
    /// A live progress update for a download or upload.
    Progress {
        /// The current transfer snapshot.
        progress: TransferProgress,
    },
    /// A transfer phase finished with aggregated statistics.
    TransferFinished {
        /// The phase that finished.
        phase: TestPhase,
        /// Final transfer statistics.
        stats: TransferStats,
    },
    /// The entire test completed successfully.
    Finished {
        /// The complete report.
        report: TestReport,
    },
    /// The test failed and cannot continue.
    Failed {
        /// A human-readable description of the failure.
        message: String,
    },
}

/// Sends an event to the consumer.
///
/// A closed channel means the consumer went away, which the engine treats
/// as a cancellation rather than an error worth surfacing.
pub(crate) async fn emit(
    events: &mpsc::Sender<EngineEvent>,
    event: EngineEvent,
) -> EngineResult<()> {
    events.send(event).await.map_err(|_| EngineError::Cancelled)
}
