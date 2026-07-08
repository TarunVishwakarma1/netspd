//! Tests for the composite connection score.

mod common;

use netspd::app::score::{CompositeScore, ScoreGrade};

#[test]
fn fast_stable_connection_grades_a_plus() {
    let report = common::ReportBuilder::new()
        .download(200.0)
        .upload(80.0)
        .ping(10.0)
        .jitter(1.0)
        .bufferbloat(10.0, 11.0, 12.0) // A+
        .build();

    let score = CompositeScore::compute(&report);
    assert_eq!(score.grade, ScoreGrade::APlus);
    assert!(score.points >= 90, "expected ≥ 90, got {}", score.points);
}

#[test]
fn slow_high_latency_grades_f() {
    let report = common::ReportBuilder::new()
        .download(1.0)
        .upload(0.5)
        .ping(200.0)
        .jitter(30.0)
        .build();

    let score = CompositeScore::compute(&report);
    assert_eq!(score.grade, ScoreGrade::F);
    assert!(score.points < 50, "expected < 50, got {}", score.points);
}

#[test]
fn mid_tier_scores_between_b_and_c() {
    let report = common::ReportBuilder::new()
        .download(25.0)
        .upload(10.0)
        .ping(60.0)
        .jitter(5.0)
        .build();

    let score = CompositeScore::compute(&report);
    assert!(
        matches!(score.grade, ScoreGrade::B | ScoreGrade::C),
        "expected B or C, got {}",
        score.grade.label()
    );
}

#[test]
fn score_without_bufferbloat_stays_within_range() {
    let report = common::ReportBuilder::new()
        .download(100.0)
        .upload(50.0)
        .ping(20.0)
        .jitter(2.0)
        .build(); // no bufferbloat

    let score = CompositeScore::compute(&report);
    assert!(score.points <= 100);
    assert!(
        score.points >= 60,
        "reasonable connection should score ≥ 60"
    );
}

#[test]
fn grade_labels_are_correct() {
    assert_eq!(ScoreGrade::APlus.label(), "A+");
    assert_eq!(ScoreGrade::A.label(), "A");
    assert_eq!(ScoreGrade::B.label(), "B");
    assert_eq!(ScoreGrade::C.label(), "C");
    assert_eq!(ScoreGrade::D.label(), "D");
    assert_eq!(ScoreGrade::F.label(), "F");
}

#[test]
fn bufferbloat_f_drags_score_down() {
    let without = common::ReportBuilder::new()
        .download(100.0)
        .upload(50.0)
        .ping(20.0)
        .jitter(2.0)
        .build();

    let with_f = common::ReportBuilder::new()
        .download(100.0)
        .upload(50.0)
        .ping(20.0)
        .jitter(2.0)
        .bufferbloat(20.0, 500.0, 600.0) // grade F
        .build();

    let score_without = CompositeScore::compute(&without);
    let score_with = CompositeScore::compute(&with_f);
    assert!(
        score_with.points < score_without.points,
        "F bufferbloat should lower the score"
    );
}
