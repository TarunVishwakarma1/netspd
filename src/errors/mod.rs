//! Error types shared across all layers of netspd.
//!
//! The engine and configuration layers expose precise, typed errors via
//! [`thiserror`], while the binary entry point aggregates them with
//! [`anyhow`]. No layer ever panics on a recoverable failure.

mod config;
mod engine;

pub use config::{ConfigError, ConfigResult};
pub use engine::{EngineError, EngineResult};
