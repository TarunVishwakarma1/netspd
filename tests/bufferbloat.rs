//! Tests for bufferbloat grading.

mod common;

use netspd::app::share::share_text;
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
    // Loaded latency below idle (jitter) never grades worse than A+.
    let bloat = Bufferbloat::new(20.0, 15.0, 18.0);
    assert_eq!(bloat.grade, BufferbloatGrade::APlus);
}

#[test]
fn share_card_includes_grade_when_measured() {
    let report = common::ReportBuilder::new()
        .download(50.0)
        .upload(20.0)
        .bufferbloat(20.0, 45.0, 30.0)
        .build();

    assert!(share_text(&report, "LibreSpeed").contains("bufferbloat A"));
}
