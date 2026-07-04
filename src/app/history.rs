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
    /// Median latency during download, in milliseconds, when measured.
    #[serde(default)]
    pub loaded_down_ms: Option<f64>,
    /// Median latency during upload, in milliseconds, when measured.
    #[serde(default)]
    pub loaded_up_ms: Option<f64>,
    /// Bufferbloat grade (`A+`..`F`), when measured.
    #[serde(default)]
    pub bufferbloat: Option<String>,
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
            loaded_down_ms: report.bufferbloat.map(|b| round1(b.download_ms)),
            loaded_up_ms: report.bufferbloat.map(|b| round1(b.upload_ms)),
            bufferbloat: report.bufferbloat.map(|b| b.grade.label().to_owned()),
        }
    }

    /// Serializes the record as a single JSON line.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    /// The CSV header matching [`HistoryRecord::to_csv_row`].
    pub const CSV_HEADER: &'static str = "timestamp,server,ping_ms,jitter_ms,packet_loss_pct,\
         download_mbps,download_peak_mbps,download_bytes,\
         upload_mbps,upload_peak_mbps,upload_bytes,\
         loaded_down_ms,loaded_up_ms,bufferbloat";

    /// Serializes the record as one CSV row; unmeasured fields stay
    /// empty.
    #[must_use]
    pub fn to_csv_row(&self) -> String {
        let opt = |value: Option<f64>| value.map(|v| v.to_string()).unwrap_or_default();
        format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            self.timestamp,
            csv_field(&self.server),
            self.ping_ms,
            self.jitter_ms,
            self.packet_loss_pct,
            self.download_mbps,
            self.download_peak_mbps,
            self.download_bytes,
            self.upload_mbps,
            self.upload_peak_mbps,
            self.upload_bytes,
            opt(self.loaded_down_ms),
            opt(self.loaded_up_ms),
            self.bufferbloat.as_deref().unwrap_or_default(),
        )
    }
}

/// Quotes a CSV field when it contains delimiters or quotes.
fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
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

/// Reads up to the last `limit` records from a history file.
///
/// Unreadable files yield an empty list; individual malformed lines are
/// skipped, so one corrupt entry never hides the rest.
#[must_use]
pub fn load(path: &Path, limit: usize) -> Vec<HistoryRecord> {
    let Ok(contents) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let records: Vec<HistoryRecord> = contents
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    let skip = records.len().saturating_sub(limit);
    records.into_iter().skip(skip).collect()
}

/// Reads up to the last `limit` records from the default history file.
#[must_use]
pub fn load_recent(limit: usize) -> Vec<HistoryRecord> {
    default_path().map_or_else(Vec::new, |path| load(&path, limit))
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}
