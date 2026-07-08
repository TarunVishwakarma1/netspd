//! Domain models shared by the engine and its consumers.
//!
//! These types are plain data: they carry no networking or UI concerns and
//! can be reused by a CLI, GUI, REST API or library front end.

mod phase;
mod progress;
mod report;
mod server;

pub use phase::TestPhase;
pub use progress::TransferProgress;
pub use report::{
    Bufferbloat, BufferbloatGrade, IpVersion, LatencyStats, TestReport, TransferStats,
};
pub use server::{Endpoints, Server};
