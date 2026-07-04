//! Tests for bufferbloat grading.

use netspd::engine::models::{Bufferbloat, BufferbloatGrade};

#[test]
fn grades_follow_waveform_thresholds() {
    assert_eq!(
        BufferbloatGrade::from_increase_ms(0.0),
        BufferbloatGrade::APlus
    );
    assert_eq!(
        BufferbloatGrade::from_increase_ms(4.9),
        BufferbloatGrade::APlus
    );
    assert_eq!(BufferbloatGrade::from_increase_ms(5.0), BufferbloatGrade::A);
    assert_eq!(
        BufferbloatGrade::from_increase_ms(30.0),
        BufferbloatGrade::B
    );
    assert_eq!(
        BufferbloatGrade::from_increase_ms(60.0),
        BufferbloatGrade::C
    );
    assert_eq!(
        BufferbloatGrade::from_increase_ms(200.0),
        BufferbloatGrade::D
    );
    assert_eq!(
        BufferbloatGrade::from_increase_ms(400.0),
        BufferbloatGrade::F
    );
}

#[test]
fn labels_render_for_display() {
    assert_eq!(BufferbloatGrade::APlus.label(), "A+");
    assert_eq!(BufferbloatGrade::F.label(), "F");
}

#[test]
fn worst_direction_drives_the_grade() {
    // Download stays flat, upload bloats badly: grade from upload.
    let bloat = Bufferbloat::new(20.0, 22.0, 250.0);
    assert_eq!(bloat.grade, BufferbloatGrade::D);
    // Loaded latency below idle (jitter) never grades better than A+.
    let bloat = Bufferbloat::new(20.0, 15.0, 18.0);
    assert_eq!(bloat.grade, BufferbloatGrade::APlus);
}

#[test]
fn share_card_includes_grade_when_measured() {
    use std::time::Duration;

    use netspd::app::share::share_text;
    use netspd::engine::models::{LatencyStats, TestReport, TransferStats};

    let stats = TransferStats {
        bytes: 1,
        duration: Duration::from_secs(1),
        average_bps: 1.0,
        peak_bps: 1.0,
    };
    let report = TestReport {
        server_name: "S".to_owned(),
        latency: LatencyStats {
            average_ms: 20.0,
            jitter_ms: 1.0,
            min_ms: 19.0,
            max_ms: 22.0,
            samples: 10,
            packet_loss_pct: 0.0,
        },
        download: stats,
        upload: stats,
        bufferbloat: Some(Bufferbloat::new(20.0, 45.0, 30.0)),
    };
    assert!(share_text(&report, "LibreSpeed").contains("bufferbloat A"));
}
