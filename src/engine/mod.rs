//! The speed test engine.
//!
//! This layer is completely UI-free: it exposes typed models, an event
//! stream and one façade type ([`Engine`]). Any front end — TUI, CLI, GUI,
//! REST API — consumes it the same way.

#[allow(clippy::module_inception)]
mod engine;
mod event;
pub mod metrics;
pub mod models;
pub mod network;
pub mod providers;
mod scheduler;

pub use engine::{Engine, EngineConfig, IpFamily, PingConfig, TransferConfig};
pub use event::EngineEvent;
