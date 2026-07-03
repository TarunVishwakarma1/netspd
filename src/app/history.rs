//! Persistent result history and JSON serialization of reports.
//!
//! Every completed test is appended as one JSON line to
//! `<data dir>/netspd/history.jsonl`, giving other tools (and future
//! trend views) a stable, machine-readable log.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::engine::models::TestReport;

/// One megabit, in bits.
const MBPS: f64 = 1_000_000.0;

/// A flat, serialization-friendly view of a [`TestReport`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryRecord {
    /// Completion time, in Unix seconds.
    pub timestamp: u64,
    /// Server the test ran against.
    pub server: String,
    /// Trimmed mean latency, in milliseconds.
    pub ping_ms: f64,
    /// Jitter, in milliseconds.
    pub jitter_ms: f64,
    /// Packet loss percentage.
    pub packet_loss_pct: f64,
    /// Average download speed, in Mbps.
    pub download_mbps: f64,
    /// Peak download speed, in Mbps.
    pub download_peak_mbps: f64,
    /// Bytes downloaded.
    pub download_bytes: u64,
    /// Average upload speed, in Mbps.
    pub upload_mbps: f64,
    /// Peak upload speed, in Mbps.
    pub upload_peak_mbps: f64,
    /// Bytes uploaded.
    pub upload_bytes: u64,
}

impl HistoryRecord {
    /// Builds a record from a completed report, stamped with the current
    /// time.
    #[must_use]
    pub fn from_report(report: &TestReport) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs());
        Self {
            timestamp,
            server: report.server_name.clone(),
            ping_ms: round1(report.latency.average_ms),
            jitter_ms: round1(report.latency.jitter_ms),
            packet_loss_pct: round1(report.latency.packet_loss_pct),
            download_mbps: round1(report.download.average_bps / MBPS),
            download_peak_mbps: round1(report.download.peak_bps / MBPS),
            download_bytes: report.download.bytes,
            upload_mbps: round1(report.upload.average_bps / MBPS),
            upload_peak_mbps: round1(report.upload.peak_bps / MBPS),
            upload_bytes: report.upload.bytes,
        }
    }

    /// Serializes the record as a single JSON line.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

/// The default history file location.
#[must_use]
pub fn default_path() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join("netspd").join("history.jsonl"))
}

/// Appends one record to a history file, creating parent directories on
/// first use.
pub fn append(path: &Path, record: &HistoryRecord) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let line = record
        .to_json()
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{line}")
}

/// Appends a completed report to the default history file.
///
/// Best-effort: history must never interrupt the application, so I/O
/// failures are swallowed.
pub fn record_report(report: &TestReport) {
    if let Some(path) = default_path() {
        let _ = append(&path, &HistoryRecord::from_report(report));
    }
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}
