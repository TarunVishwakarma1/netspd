//! Tests for the plain-language verdict.

mod common;

use netspd::app::verdict::verdict;

#[test]
fn fast_stable_connection_reads_positive() {
    let report = common::ReportBuilder::new()
        .download(95.0)
        .upload(88.0)
        .ping(20.0)
        .bufferbloat(20.0, 22.0, 24.0)
        .build();

    let text = verdict(&report);
    assert!(text.contains("Great for 4K streaming"));
    assert!(text.contains("video calls should be smooth"));
    assert!(text.contains("responsive for online gaming"));
}

#[test]
fn bufferbloat_warns_about_calls_and_gaming() {
    let report = common::ReportBuilder::new()
        .download(95.0)
        .upload(88.0)
        .ping(20.0)
        .bufferbloat(20.0, 450.0, 500.0) // grade F
        .build();

    let text = verdict(&report);
    assert!(text.contains("video calls may stutter"));
    assert!(text.contains("lag spikes"));
}

#[test]
fn slow_link_reads_honest() {
    let report = common::ReportBuilder::new()
        .download(2.0)
        .upload(0.8)
        .ping(180.0)
        .build();

    let text = verdict(&report);
    assert!(text.contains("Too slow for smooth video streaming"));
    assert!(text.contains("upload is tight for video calls"));
    assert!(text.contains("high ping"));
}

#[test]
fn mid_tier_connection() {
    let report = common::ReportBuilder::new()
        .download(15.0)
        .upload(5.0)
        .ping(80.0)
        .build();

    let text = verdict(&report);
    assert!(text.contains("Good for HD streaming"));
    assert!(text.contains("playable for casual gaming"));
}

#[test]
fn verdict_does_not_mention_loss_when_zero() {
    let report = common::ReportBuilder::new()
        .download(100.0)
        .upload(50.0)
        .ping(10.0)
        .loss(0.0)
        .build();

    assert!(!verdict(&report).contains("packet loss"));
}
