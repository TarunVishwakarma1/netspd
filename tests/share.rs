//! Tests for the shareable result card.

use std::time::Duration;

use netspd::app::share::share_text;
use netspd::engine::models::{LatencyStats, TestReport, TransferStats};

#[test]
fn share_card_contains_the_headline_figures() {
    let report = TestReport {
        server_name: "Tokyo, Japan (A573)".to_owned(),
        latency: LatencyStats {
            average_ms: 141.2,
            jitter_ms: 1.0,
            min_ms: 138.0,
            max_ms: 150.0,
            samples: 10,
            packet_loss_pct: 0.0,
        },
        download: TransferStats {
            bytes: 100_000_000,
            duration: Duration::from_secs(10),
            average_bps: 104_900_000.0,
            peak_bps: 206_500_000.0,
        },
        upload: TransferStats {
            bytes: 40_000_000,
            duration: Duration::from_secs(10),
            average_bps: 36_300_000.0,
            peak_bps: 51_900_000.0,
        },
        bufferbloat: None,
    };
    let card = share_text(&report, "LibreSpeed");
    // Without a bufferbloat measurement the card omits the grade.
    assert!(!card.contains("bufferbloat"));
    assert!(card.contains("Tokyo, Japan (A573)"));
    assert!(card.contains("↓ 104.9 Mbps"));
    assert!(card.contains("↑ 36.3 Mbps"));
    assert!(card.contains("ping 141 ms"));
    assert!(card.contains("loss 0%"));
    assert!(card.contains("via LibreSpeed"));
    // Compact: three lines, ready for a chat message.
    assert_eq!(card.lines().count(), 3);
}
