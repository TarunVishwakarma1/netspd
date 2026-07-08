//! Tests for the speed-tier color mapping on `Colors`.

use netspd::tui::theme::{Colors, Theme};

type R = Result<(), Box<dyn std::error::Error>>;

fn default_colors() -> Result<Colors, Box<dyn std::error::Error>> {
    let themes = Theme::load_all(None)?;
    themes
        .into_iter()
        .next()
        .ok_or_else(|| "no built-in themes".into())
        .map(|t| t.colors)
}

#[test]
fn above_100_mbps_is_success() -> R {
    let c = default_colors()?;
    assert_eq!(c.speed_color(100.0 * 1_000_000.0), c.success);
    assert_eq!(c.speed_color(500.0 * 1_000_000.0), c.success);
    Ok(())
}

#[test]
fn between_25_and_100_mbps_is_warning() -> R {
    let c = default_colors()?;
    assert_eq!(c.speed_color(25.0 * 1_000_000.0), c.warning);
    assert_eq!(c.speed_color(99.9 * 1_000_000.0), c.warning);
    Ok(())
}

#[test]
fn below_25_mbps_is_danger() -> R {
    let c = default_colors()?;
    assert_eq!(c.speed_color(0.0), c.danger);
    assert_eq!(c.speed_color(24.9 * 1_000_000.0), c.danger);
    Ok(())
}

#[test]
fn boundary_exactly_100_mbps_is_success() -> R {
    let c = default_colors()?;
    assert_eq!(c.speed_color(100.0 * 1_000_000.0), c.success);
    assert_eq!(c.speed_color(99.999 * 1_000_000.0), c.warning);
    Ok(())
}
