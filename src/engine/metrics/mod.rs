//! Reusable, independently-testable metric calculators.
//!
//! Nothing in this module touches the network or the UI; every calculator
//! is a pure state machine driven by explicit inputs, which keeps the whole
//! module trivially unit-testable.

mod ema;
mod latency;
mod sampler;
pub mod statistics;
mod throughput;

pub use ema::Ema;
pub use latency::LatencyCalculator;
pub use sampler::Sampler;
pub use throughput::{ThroughputMeter, ThroughputSnapshot};
