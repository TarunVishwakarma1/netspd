//! Errors produced by the speed test engine.

use thiserror::Error;

/// Convenient result alias for engine operations.
pub type EngineResult<T> = Result<T, EngineError>;

/// Every failure mode the engine can encounter.
///
/// Network failures never panic: they are surfaced through this type and
/// translated into user-facing events by the application layer.
#[derive(Debug, Error)]
pub enum EngineError {
    /// An HTTP request failed (connection, TLS, timeout, body errors).
    #[error("network request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// A server definition contained an unusable URL.
    #[error("invalid server URL: {0}")]
    InvalidUrl(String),

    /// A server response was malformed or failed validation.
    #[error("invalid server response: {0}")]
    InvalidResponse(String),

    /// Server discovery returned no reachable servers.
    #[error("no reachable speed test servers (check your connection or configured servers)")]
    NoServers,

    /// A measurement phase produced no samples (e.g. every ping failed).
    #[error("no measurement samples were collected: {reason}")]
    NoSamples {
        /// Why the last sample attempt failed (TLS error, timeout, HTTP
        /// status, …) so dead servers are diagnosable from the UI.
        reason: String,
    },

    /// The test was cancelled by the user.
    #[error("test cancelled")]
    Cancelled,

    /// A background worker task failed to join.
    #[error("worker task failed: {0}")]
    Task(#[from] tokio::task::JoinError),
}

impl EngineError {
    /// Returns `true` when this error represents a user-initiated
    /// cancellation rather than a genuine failure.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }
}
