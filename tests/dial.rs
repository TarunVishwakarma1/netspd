//! Tests for the dial gauge: pure scale/angle math and a rendered
//! smoke test against ratatui's [`TestBackend`].

use ratatui::backend::TestBackend;
use ratatui::style::Color;
use ratatui::Terminal;

use netspd::tui::theme::Theme;
use netspd::tui::widgets::dial_gauge::{band_color, nice_max_bps, value_angle_deg};

const MBPS: f64 = 1_000_000.0;

#[test]
fn scale_has_a_floor_of_100_mbps() {
    assert_eq!(nice_max_bps(0.0), 100.0 * MBPS);
    assert_eq!(nice_max_bps(3.0 * MBPS), 100.0 * MBPS);
    assert_eq!(nice_max_bps(80.0 * MBPS), 100.0 * MBPS);
}

#[test]
fn scale_bumps_to_next_round_step() {
    assert_eq!(nice_max_bps(120.0 * MBPS), 250.0 * MBPS);
    assert_eq!(nice_max_bps(260.0 * MBPS), 500.0 * MBPS);
    assert_eq!(nice_max_bps(600.0 * MBPS), 1000.0 * MBPS);
    assert_eq!(nice_max_bps(1_200.0 * MBPS), 2500.0 * MBPS);
}

#[test]
fn scale_never_shrinks_as_peak_grows() {
    let mut previous = 0.0;
    for peak_mbps in (0..3000).step_by(50) {
        let scale = nice_max_bps(f64::from(peak_mbps) * MBPS);
        assert!(scale >= previous, "scale shrank at {peak_mbps} Mbps");
        assert!(scale >= f64::from(peak_mbps) * MBPS, "needle would pin");
        previous = scale;
    }
}

#[test]
fn angle_covers_240_degree_sweep() {
    assert!((value_angle_deg(0.0) - 210.0).abs() < 1e-9);
    assert!((value_angle_deg(0.5) - 90.0).abs() < 1e-9);
    assert!((value_angle_deg(1.0) - (-30.0)).abs() < 1e-9);
}

#[test]
fn angle_clamps_out_of_range_ratios() {
    assert!((value_angle_deg(-0.5) - 210.0).abs() < 1e-9);
    assert!((value_angle_deg(2.0) - (-30.0)).abs() < 1e-9);
}

#[test]
fn band_stays_cool_early_and_heats_to_danger() {
    let phase = Color::Rgb(100, 200, 255);
    let danger = Color::Rgb(255, 0, 0);
    assert_eq!(band_color(0.0, phase, danger), phase);
    assert_eq!(band_color(0.5, phase, danger), phase);
    assert_eq!(band_color(1.0, phase, danger), danger);
}

#[test]
fn dial_renders_braille_art_with_scale_labels() -> Result<(), Box<dyn std::error::Error>> {
    let themes = Theme::builtin()?;
    let Some(theme) = themes.first() else {
        return Err("no builtin theme".into());
    };
    let mut terminal = Terminal::new(TestBackend::new(90, 24))?;
    let trail = [90_000_000.0, 110_000_000.0, 135_000_000.0];
    terminal.draw(|frame| {
        netspd::tui::widgets::dial_gauge::render(
            frame,
            frame.area(),
            theme,
            &netspd::tui::widgets::dial_gauge::DialData {
                label: "Download",
                bps: 142_500_000.0,
                color: theme.colors.download,
                peak_bps: 210_000_000.0,
                trail: &trail,
                ping_ms: Some(23.0),
                override_ratio: None,
                dimmed: false,
                speed_unit: netspd::utils::format::SpeedUnit::Bits,
            },
        );
    })?;

    let buffer = terminal.backend().buffer();
    let mut art = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            art.push_str(buffer[(x, y)].symbol());
        }
        art.push('\n');
    }
    println!("{art}");

    // Braille dots must be present (the arc, band and needle).
    assert!(art
        .chars()
        .any(|ch| ('\u{2800}'..='\u{28ff}').contains(&ch)));
    // Auto-scale for a 210 Mbps peak is 250 Mbps: labels 0..250 inside.
    for label in ["0", "50", "100", "150", "200", "250"] {
        assert!(art.contains(label), "missing tick label {label}");
    }
    // Face captions, ping inset and readout. On a 24-row dial the value
    // renders in the block-digit font, so check for the digit `4`'s
    // middle glyph row rather than plain text.
    assert!(art.contains("DOWNLOAD"));
    assert!(art.contains("Mbps"));
    assert!(art.contains("23ms"));
    assert!(art.contains("142.5") || art.contains("█▄▄█"));
    Ok(())
}
