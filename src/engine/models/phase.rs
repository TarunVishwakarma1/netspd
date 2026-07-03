//! The phases of a complete speed test.

/// A single phase within a speed test run.
///
/// Phases always execute in the order: ping, download, upload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestPhase {
    /// Latency and jitter measurement.
    Ping,
    /// Download throughput measurement.
    Download,
    /// Upload throughput measurement.
    Upload,
}

impl TestPhase {
    /// Human-readable label for this phase.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Ping => "Ping",
            Self::Download => "Download",
            Self::Upload => "Upload",
        }
    }

    /// The phase that follows this one, if any.
    #[must_use]
    pub fn next(self) -> Option<Self> {
        match self {
            Self::Ping => Some(Self::Download),
            Self::Download => Some(Self::Upload),
            Self::Upload => None,
        }
    }
}
