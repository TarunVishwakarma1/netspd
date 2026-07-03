//! Async networking primitives: HTTP client, ping, download and upload.
//!
//! This module knows nothing about providers or the UI. It receives plain
//! URLs and configuration, streams data without buffering whole payloads,
//! and reports through typed events with graceful cancellation throughout.

pub mod client;
pub mod download;
pub mod health;
pub mod info;
pub mod ping;
mod transfer;
pub mod upload;
