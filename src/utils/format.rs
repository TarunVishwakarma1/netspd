//! Human-friendly formatting of speeds, sizes, durations and latencies.

use std::time::Duration;

const BPS_UNITS: [&str; 5] = ["bps", "Kbps", "Mbps", "Gbps", "Tbps"];
const BYTE_UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
// MB/s units scale in binary-like steps: B/s, KB/s, MB/s, GB/s, TB/s
const BPS_BYTE_UNITS: [&str; 5] = ["B/s", "KB/s", "MB/s", "GB/s", "TB/s"];
const STEP: f64 = 1000.0;

/// Whether to display speeds in bits per second or bytes per second.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpeedUnit {
    /// Display as Mbps / Gbps (default, industry-standard for ISPs).
    #[default]
    Bits,
    /// Display as MB/s / GB/s (preferred by some power users).
    Bytes,
}

impl SpeedUnit {
    /// Cycle to the next unit.
    #[must_use]
    pub fn toggle(self) -> Self {
        match self {
            Self::Bits => Self::Bytes,
            Self::Bytes => Self::Bits,
        }
    }
}

/// Formats a bits-per-second value as a compact human string,
/// e.g. `142.5 Mbps`.
#[must_use]
pub fn format_bps(bps: f64) -> String {
    let (value, unit) = scale(bps.max(0.0), &BPS_UNITS);
    format!("{value:.1} {unit}")
}

/// Splits a bits-per-second value into a numeric string and its unit
/// label, respecting the requested [`SpeedUnit`].
///
/// Examples:
/// - `Bits`:  `("142.5", "Mbps")`
/// - `Bytes`: `("17.8",  "MB/s")`
#[must_use]
pub fn split_bps_unit(bps: f64, unit: SpeedUnit) -> (String, &'static str) {
    match unit {
        SpeedUnit::Bits => {
            let (value, label) = scale(bps.max(0.0), &BPS_UNITS);
            (format!("{value:.1}"), label)
        }
        SpeedUnit::Bytes => {
            let bytes_per_sec = bps.max(0.0) / 8.0;
            let (value, label) = scale(bytes_per_sec, &BPS_BYTE_UNITS);
            (format!("{value:.1}"), label)
        }
    }
}

/// Splits a bits-per-second value into `("142.5", "Mbps")`.
/// Equivalent to `split_bps_unit(bps, SpeedUnit::Bits)`.
#[must_use]
pub fn split_bps(bps: f64) -> (String, &'static str) {
    split_bps_unit(bps, SpeedUnit::Bits)
}

/// Formats a byte count as a compact human string, e.g. `1.24 GB`.
#[must_use]
pub fn format_bytes(bytes: u64) -> String {
    let (value, unit) = scale(bytes as f64, &BYTE_UNITS);
    if unit == "B" {
        format!("{bytes} B")
    } else {
        format!("{value:.2} {unit}")
    }
}

/// Formats a duration as `mm:ss`.
#[must_use]
pub fn format_duration(duration: Duration) -> String {
    let total = duration.as_secs();
    format!("{:02}:{:02}", total / 60, total % 60)
}

/// Formats a duration as a short countdown, e.g. `8s` or `1m 05s`.
#[must_use]
pub fn format_eta(duration: Duration) -> String {
    let total = duration.as_secs();
    if total >= 60 {
        format!("{}m {:02}s", total / 60, total % 60)
    } else {
        format!("{total}s")
    }
}

/// Formats a latency value in milliseconds, e.g. `12.4 ms`.
#[must_use]
pub fn format_millis(ms: f64) -> String {
    if ms >= 100.0 {
        format!("{ms:.0} ms")
    } else {
        format!("{ms:.1} ms")
    }
}

/// Scales a raw value into the largest unit that keeps it below 1000.
fn scale(value: f64, units: &[&'static str; 5]) -> (f64, &'static str) {
    let mut scaled = value;
    let mut index = 0;
    while scaled >= STEP && index < units.len() - 1 {
        scaled /= STEP;
        index += 1;
    }
    (scaled, units[index])
}
