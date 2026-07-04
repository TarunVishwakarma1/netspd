//! Theme definitions, TOML parsing and color utilities.
//!
//! Themes are plain TOML files; the five built-in themes are embedded in
//! the binary, and user themes are loaded from disk at startup — changing
//! themes never requires recompiling.

use std::path::Path;

use ratatui::style::Color;
use serde::Deserialize;

use crate::errors::{ConfigError, ConfigResult};

/// A complete color palette plus identity for one theme.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Display name, e.g. `Nord`.
    pub name: String,
    /// The resolved palette.
    pub colors: Colors,
}

/// Every color a widget can ask for.
#[derive(Debug, Clone, Copy)]
pub struct Colors {
    /// Application background.
    pub background: Color,
    /// Card / panel background.
    pub surface: Color,
    /// Popup / overlay background.
    pub overlay: Color,
    /// Primary text.
    pub text: Color,
    /// Secondary text.
    pub subtext: Color,
    /// De-emphasized text and separators.
    pub muted: Color,
    /// Borders and outlines.
    pub border: Color,
    /// Primary accent.
    pub accent: Color,
    /// Secondary accent, used for gradients.
    pub accent_alt: Color,
    /// Positive results.
    pub success: Color,
    /// Cautionary values.
    pub warning: Color,
    /// Errors and failures.
    pub danger: Color,
    /// Download-phase highlight.
    pub download: Color,
    /// Upload-phase highlight.
    pub upload: Color,
    /// Latency highlight.
    pub latency: Color,
}

/// Serde mirror of a theme file, with colors as hex strings.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTheme {
    name: String,
    colors: RawColors,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawColors {
    background: String,
    surface: String,
    overlay: String,
    text: String,
    subtext: String,
    muted: String,
    border: String,
    accent: String,
    accent_alt: String,
    success: String,
    warning: String,
    danger: String,
    download: String,
    upload: String,
    latency: String,
}

impl Theme {
    /// Parses a theme from TOML source.
    pub fn from_toml(source: &str) -> ConfigResult<Self> {
        let raw: RawTheme =
            toml::from_str(source).map_err(|err| ConfigError::InvalidTheme(Box::new(err)))?;
        Ok(Self {
            name: raw.name,
            colors: Colors {
                background: parse_hex(&raw.colors.background)?,
                surface: parse_hex(&raw.colors.surface)?,
                overlay: parse_hex(&raw.colors.overlay)?,
                text: parse_hex(&raw.colors.text)?,
                subtext: parse_hex(&raw.colors.subtext)?,
                muted: parse_hex(&raw.colors.muted)?,
                border: parse_hex(&raw.colors.border)?,
                accent: parse_hex(&raw.colors.accent)?,
                accent_alt: parse_hex(&raw.colors.accent_alt)?,
                success: parse_hex(&raw.colors.success)?,
                warning: parse_hex(&raw.colors.warning)?,
                danger: parse_hex(&raw.colors.danger)?,
                download: parse_hex(&raw.colors.download)?,
                upload: parse_hex(&raw.colors.upload)?,
                latency: parse_hex(&raw.colors.latency)?,
            },
        })
    }

    /// The five themes embedded in the binary.
    pub fn builtin() -> ConfigResult<Vec<Self>> {
        [
            include_str!("../../assets/themes/default.toml"),
            include_str!("../../assets/themes/nord.toml"),
            include_str!("../../assets/themes/dracula.toml"),
            include_str!("../../assets/themes/catppuccin.toml"),
            include_str!("../../assets/themes/gruvbox.toml"),
        ]
        .into_iter()
        .map(Self::from_toml)
        .collect()
    }

    /// Loads all themes: built-ins first, then any `*.toml` files from
    /// `user_dir`. Unreadable or invalid user themes are skipped silently
    /// so a broken file never blocks startup.
    pub fn load_all(user_dir: Option<&Path>) -> ConfigResult<Vec<Self>> {
        let mut themes = Self::builtin()?;
        if let Some(dir) = user_dir {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|ext| ext == "toml") {
                        if let Ok(source) = std::fs::read_to_string(&path) {
                            if let Ok(theme) = Self::from_toml(&source) {
                                themes.push(theme);
                            }
                        }
                    }
                }
            }
        }
        Ok(themes)
    }
}

/// Parses a `#rrggbb` hex string into an RGB color.
pub fn parse_hex(value: &str) -> ConfigResult<Color> {
    let hex = value.trim().trim_start_matches('#');
    if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(ConfigError::InvalidColor(value.to_owned()));
    }
    let component = |range: std::ops::Range<usize>| -> ConfigResult<u8> {
        u8::from_str_radix(&hex[range], 16).map_err(|_| ConfigError::InvalidColor(value.to_owned()))
    };
    Ok(Color::Rgb(
        component(0..2)?,
        component(2..4)?,
        component(4..6)?,
    ))
}

impl Theme {
    /// Strips all colors for `NO_COLOR` terminals, keeping structure
    /// legible through the terminal's own default foreground.
    #[must_use]
    pub fn without_colors(&self) -> Self {
        let c = Color::Reset;
        Self {
            name: self.name.clone(),
            colors: Colors {
                background: c,
                surface: c,
                overlay: c,
                text: c,
                subtext: c,
                muted: c,
                border: c,
                accent: c,
                accent_alt: c,
                success: c,
                warning: c,
                danger: c,
                download: c,
                upload: c,
                latency: c,
            },
        }
    }
}

/// Linearly blends two RGB colors; `t` is clamped to `0.0..=1.0`.
///
/// Non-RGB colors cannot be blended and fall back to `a`.
#[must_use]
pub fn blend(a: Color, b: Color, t: f64) -> Color {
    match (a, b) {
        (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
            let t = t.clamp(0.0, 1.0);
            let mix = |x: u8, y: u8| -> u8 {
                (f64::from(x) + (f64::from(y) - f64::from(x)) * t).round() as u8
            };
            Color::Rgb(mix(r1, r2), mix(g1, g2), mix(b1, b2))
        }
        _ => a,
    }
}
