//! Tests for theme parsing.

use netspd::tui::theme::{blend, parse_hex, Theme};
use ratatui::style::Color;

#[test]
fn all_builtin_themes_parse() {
    let themes = Theme::builtin().unwrap_or_default();
    // Themes are discovered alphabetically by filename via build.rs.
    // Update this list when a new theme file is added to assets/themes/.
    let mut names: Vec<&str> = themes.iter().map(|t| t.name.as_str()).collect();
    names.sort();
    assert!(
        names.contains(&"Default"),
        "expected Default theme, got: {names:?}"
    );
    assert!(
        names.contains(&"Nord"),
        "expected Nord theme, got: {names:?}"
    );
    // All files must parse without error.
    assert!(!names.is_empty());
}

#[test]
fn hex_parsing_accepts_valid_colors() {
    assert_eq!(
        parse_hex("#2e3440").ok(),
        Some(Color::Rgb(0x2e, 0x34, 0x40))
    );
    assert_eq!(parse_hex("ffffff").ok(), Some(Color::Rgb(255, 255, 255)));
}

#[test]
fn hex_parsing_rejects_invalid_colors() {
    assert!(parse_hex("#fff").is_err());
    assert!(parse_hex("#zzzzzz").is_err());
    assert!(parse_hex("").is_err());
}

#[test]
fn incomplete_theme_is_rejected() {
    let source = r##"
        name = "Broken"
        [colors]
        background = "#000000"
    "##;
    assert!(Theme::from_toml(source).is_err());
}

#[test]
fn blend_interpolates_rgb() {
    let a = Color::Rgb(0, 0, 0);
    let b = Color::Rgb(100, 200, 50);
    assert_eq!(blend(a, b, 0.5), Color::Rgb(50, 100, 25));
    assert_eq!(blend(a, b, 0.0), a);
    assert_eq!(blend(a, b, 1.0), b);
    // Non-RGB colors fall back to the first color.
    assert_eq!(blend(Color::Red, b, 0.5), Color::Red);
}
