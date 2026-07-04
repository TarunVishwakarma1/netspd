//! Tests for the plain-language verdict.

use std::time::Duration;

use netspd::app::verdict::verdict;
use netspd::engine::models::{Bufferbloat, LatencyStats, TestReport, TransferStats};

fn report(down_mbps: f64, up_mbps: f64, ping_ms: f64, bloat: Option<Bufferbloat>) -> TestReport {
    let stats = |mbps: f64| TransferStats {
        bytes: 1,
        duration: Duration::from_secs(1),
        average_bps: mbps * 1_000_000.0,
        peak_bps: mbps * 1_000_000.0,
    };
    TestReport {
        server_name: "S".to_owned(),
        latency: LatencyStats {
            average_ms: ping_ms,
            jitter_ms: 1.0,
            min_ms: ping_ms,
            max_ms: ping_ms,
            samples: 10,
            packet_loss_pct: 0.0,
        },
        download: stats(down_mbps),
        upload: stats(up_mbps),
        bufferbloat: bloat,
    }
}

#[test]
fn fast_stable_connection_reads_positive() {
    let text = verdict(&report(
        95.0,
        88.0,
        20.0,
        Some(Bufferbloat::new(20.0, 22.0, 24.0)),
    ));
    assert!(text.contains("Great for 4K streaming"));
    assert!(text.contains("video calls should be smooth"));
    assert!(text.contains("responsive for online gaming"));
}

#[test]
fn bufferbloat_warns_about_calls_and_gaming() {
    let bloat = Bufferbloat::new(20.0, 450.0, 500.0); // grade F
    let text = verdict(&report(95.0, 88.0, 20.0, Some(bloat)));
    assert!(text.contains("video calls may stutter"));
    assert!(text.contains("lag spikes"));
}

#[test]
fn slow_link_reads_honest() {
    let text = verdict(&report(2.0, 0.8, 180.0, None));
    assert!(text.contains("Too slow for smooth video streaming"));
    assert!(text.contains("upload is tight for video calls"));
    assert!(text.contains("high ping"));
}

#[test]
fn mid_tier_connection() {
    let text = verdict(&report(15.0, 5.0, 80.0, None));
    assert!(text.contains("Good for HD streaming"));
    assert!(text.contains("playable for casual gaming"));
}
