//! Composite connection score: a single 0–100 number and an A+–F grade
//! derived from download, upload, latency, jitter and bufferbloat.
//!
//! Weights:  download 30 %  ·  upload 20 %  ·  ping 25 %
//!           jitter 10 %   ·  bufferbloat 15 %
//!
//! When bufferbloat was not measured the 15 % is redistributed evenly
//! across the other four components so the score stays comparable.

use crate::engine::models::{BufferbloatGrade, TestReport};
use crate::tui::theme::Colors;
use ratatui::style::Color;

/// The computed composite score for a completed test.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompositeScore {
    /// Numeric score, 0–100.
    pub points: u8,
    /// Letter grade derived from [`CompositeScore::points`].
    pub grade: ScoreGrade,
}

impl CompositeScore {
    /// Computes the composite score for a completed test.
    #[must_use]
    pub fn compute(report: &TestReport) -> Self {
        let dl = component_download(report.download.average_bps / 1_000_000.0);
        let ul = component_upload(report.upload.average_bps / 1_000_000.0);
        let ping = component_ping(report.latency.average_ms);
        let jitter = component_jitter(report.latency.jitter_ms);

        let raw = match report.bufferbloat.map(|b| b.grade) {
            Some(grade) => {
                let bloat = component_bloat(grade);
                dl * 0.30 + ul * 0.20 + ping * 0.25 + jitter * 0.10 + bloat * 0.15
            }
            None => {
                // Redistribute the 15 % evenly: dl→34.6%, ul→23.1%, ping→28.8%, jitter→11.5%.
                dl * 0.346 + ul * 0.231 + ping * 0.288 + jitter * 0.115
            }
        };

        let points = raw.round().clamp(0.0, 100.0) as u8;
        Self {
            points,
            grade: ScoreGrade::from_points(points),
        }
    }
}

/// Letter grade for the composite score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreGrade {
    /// 90–100.
    APlus,
    /// 80–89.
    A,
    /// 70–79.
    B,
    /// 60–69.
    C,
    /// 50–59.
    D,
    /// 0–49.
    F,
}

impl ScoreGrade {
    /// Short label, e.g. `"A+"`.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::APlus => "A+",
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::F => "F",
        }
    }

    /// Theme color for this grade.
    #[must_use]
    pub fn color(self, colors: &Colors) -> Color {
        match self {
            Self::APlus | Self::A => colors.success,
            Self::B | Self::C => colors.warning,
            Self::D | Self::F => colors.danger,
        }
    }

    fn from_points(points: u8) -> Self {
        match points {
            90..=100 => Self::APlus,
            80..=89 => Self::A,
            70..=79 => Self::B,
            60..=69 => Self::C,
            50..=59 => Self::D,
            _ => Self::F,
        }
    }
}

// ── Component scorers (0 – 100) ───────────────────────────────────────────────

fn component_download(mbps: f64) -> f64 {
    if mbps >= 100.0 {
        100.0
    } else if mbps >= 50.0 {
        85.0
    } else if mbps >= 25.0 {
        70.0
    } else if mbps >= 10.0 {
        50.0
    } else if mbps >= 3.0 {
        30.0
    } else {
        10.0
    }
}

fn component_upload(mbps: f64) -> f64 {
    if mbps >= 50.0 {
        100.0
    } else if mbps >= 25.0 {
        85.0
    } else if mbps >= 10.0 {
        70.0
    } else if mbps >= 3.0 {
        50.0
    } else if mbps >= 1.0 {
        30.0
    } else {
        10.0
    }
}

fn component_ping(ms: f64) -> f64 {
    if ms < 20.0 {
        100.0
    } else if ms < 50.0 {
        85.0
    } else if ms < 80.0 {
        70.0
    } else if ms < 110.0 {
        55.0
    } else if ms < 150.0 {
        35.0
    } else {
        15.0
    }
}

fn component_jitter(ms: f64) -> f64 {
    if ms < 2.0 {
        100.0
    } else if ms < 5.0 {
        80.0
    } else if ms < 10.0 {
        60.0
    } else if ms < 20.0 {
        40.0
    } else {
        20.0
    }
}

fn component_bloat(grade: BufferbloatGrade) -> f64 {
    match grade {
        BufferbloatGrade::APlus => 100.0,
        BufferbloatGrade::A => 90.0,
        BufferbloatGrade::B => 75.0,
        BufferbloatGrade::C => 50.0,
        BufferbloatGrade::D => 25.0,
        BufferbloatGrade::F => 0.0,
    }
}
