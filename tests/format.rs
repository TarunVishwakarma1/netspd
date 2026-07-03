//! Unit tests for human-readable formatting.

use std::time::Duration;

use netspd::utils::format::{
    format_bps, format_bytes, format_duration, format_eta, format_millis, split_bps,
};

#[test]
fn bps_scales_units() {
    assert_eq!(format_bps(500.0), "500.0 bps");
    assert_eq!(format_bps(1_500.0), "1.5 Kbps");
    assert_eq!(format_bps(142_500_000.0), "142.5 Mbps");
    assert_eq!(format_bps(2_400_000_000.0), "2.4 Gbps");
}

#[test]
fn bps_never_negative() {
    assert_eq!(format_bps(-10.0), "0.0 bps");
}

#[test]
fn split_bps_separates_value_and_unit() {
    let (value, unit) = split_bps(142_500_000.0);
    assert_eq!(value, "142.5");
    assert_eq!(unit, "Mbps");
}

#[test]
fn bytes_scale_units() {
    assert_eq!(format_bytes(512), "512 B");
    assert_eq!(format_bytes(1_500), "1.50 KB");
    assert_eq!(format_bytes(1_240_000_000), "1.24 GB");
}

#[test]
fn durations_render_as_minutes_seconds() {
    assert_eq!(format_duration(Duration::from_secs(0)), "00:00");
    assert_eq!(format_duration(Duration::from_secs(83)), "01:23");
}

#[test]
fn eta_renders_compactly() {
    assert_eq!(format_eta(Duration::from_secs(8)), "8s");
    assert_eq!(format_eta(Duration::from_secs(65)), "1m 05s");
}

#[test]
fn millis_adapt_precision() {
    assert_eq!(format_millis(12.44), "12.4 ms");
    assert_eq!(format_millis(234.6), "235 ms");
}
