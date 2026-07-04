//! Plain-language verdicts: what the numbers mean for real activities.
//!
//! Nobody outside networking knows whether "141 ms, bufferbloat C" is
//! good. This module translates a report into activity statements —
//! streaming, video calls, gaming, uploads — using widely published
//! bandwidth/latency requirements (Netflix, Zoom, and Waveform's
//! bufferbloat guidance).

use crate::engine::models::{BufferbloatGrade, TestReport};

/// One megabit, in bits.
const MBPS: f64 = 1_000_000.0;

/// Builds a short, human verdict for a completed test.
///
/// Always returns at least one statement; typically two clauses joined
/// with `·`.
#[must_use]
pub fn verdict(report: &TestReport) -> String {
    let down = report.download.average_bps / MBPS;
    let up = report.upload.average_bps / MBPS;
    let ping = report.latency.average_ms;
    let bloat = report.bufferbloat.map(|b| b.grade);

    let mut clauses: Vec<String> = Vec::new();

    // Streaming, by sustained download.
    clauses.push(match down {
        d if d >= 75.0 => "Great for 4K streaming on several devices".to_owned(),
        d if d >= 25.0 => "Good for 4K streaming".to_owned(),
        d if d >= 10.0 => "Good for HD streaming".to_owned(),
        d if d >= 3.0 => "OK for SD streaming; HD may buffer".to_owned(),
        _ => "Too slow for smooth video streaming".to_owned(),
    });

    // Video calls need symmetric headroom and stable loaded latency.
    let calls_bandwidth = down >= 3.5 && up >= 3.5;
    let calls_stable = !matches!(
        bloat,
        Some(BufferbloatGrade::C | BufferbloatGrade::D | BufferbloatGrade::F)
    );
    clauses.push(match (calls_bandwidth, calls_stable) {
        (true, true) => "video calls should be smooth".to_owned(),
        (true, false) => "video calls may stutter while others use the connection".to_owned(),
        (false, _) => "upload is tight for video calls".to_owned(),
    });

    // Gaming cares about latency, then about latency under load.
    let gaming = match ping {
        p if p < 60.0 => Some("responsive for online gaming"),
        p if p < 110.0 => Some("playable for casual gaming"),
        _ => Some("high ping for fast-paced gaming"),
    };
    if let Some(gaming) = gaming {
        let suffix = match bloat {
            Some(BufferbloatGrade::D | BufferbloatGrade::F) => {
                " (expect lag spikes during downloads)"
            }
            _ => "",
        };
        clauses.push(format!("{gaming}{suffix}"));
    }

    clauses.join(" · ")
}
