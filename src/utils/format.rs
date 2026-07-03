//! Human-friendly formatting of speeds, sizes, durations and latencies.

use std::time::Duration;

const BPS_UNITS: [&str; 5] = ["bps", "Kbps", "Mbps", "Gbps", "Tbps"];
const BYTE_UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
const STEP: f64 = 1000.0;

/// Formats a bits-per-second value as a compact human string,
/// e.g. `142.5 Mbps`.
#[must_use]
pub fn format_bps(bps: f64) -> String {
    let (value, unit) = scale(bps.max(0.0), &BPS_UNITS);
    format!("{value:.1} {unit}")
}

/// Splits a bits-per-second value into a numeric string and its unit,
/// e.g. `("142.5", "Mbps")`. Useful when the UI styles them separately.
#[must_use]
pub fn split_bps(bps: f64) -> (String, &'static str) {
    let (value, unit) = scale(bps.max(0.0), &BPS_UNITS);
    (format!("{value:.1}"), unit)
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
