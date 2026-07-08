//! Tests for the shareable result card.

mod common;

use netspd::app::share::share_text;

#[test]
fn share_card_contains_the_headline_figures() {
    let report = common::ReportBuilder::new()
        .server("Tokyo, Japan (A573)")
        .download(104.9)
        .upload(36.3)
        .ping(141.2)
        .jitter(1.0)
        .loss(0.0)
        .build();

    let card = share_text(&report, "LibreSpeed");
    assert!(
        !card.contains("bufferbloat"),
        "no grade without measurement"
    );
    assert!(card.contains("Tokyo, Japan (A573)"));
    assert!(card.contains("↓ 104.9 Mbps"));
    assert!(card.contains("↑ 36.3 Mbps"));
    assert!(card.contains("ping 141 ms"));
    assert!(card.contains("loss 0%"));
    assert!(card.contains("via LibreSpeed"));
    assert!(card.contains("4K streaming"), "verdict should mention 4K");
    assert_eq!(card.lines().count(), 4, "compact four-line card");
}

#[test]
fn share_card_includes_bufferbloat_grade() {
    let report = common::ReportBuilder::new()
        .download(150.0)
        .upload(80.0)
        .bufferbloat(20.0, 45.0, 30.0)
        .build();

    let card = share_text(&report, "Ookla");
    assert!(card.contains("bufferbloat"));
}
