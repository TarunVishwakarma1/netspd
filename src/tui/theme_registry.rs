//! The theme registry: the unified collection of themes.
//!
//! [`ThemeRegistry`] is the single source of truth for all themes in the
//! application. It owns the TOML-backed themes and exposes a clean,
//! index-based API so the renderer, selector screen and application state
//! only ever track a single `usize`.

use std::path::Path;

use crate::errors::ConfigResult;

use super::theme::Theme;

/// The theme store, in stable index order.
#[derive(Debug, Clone)]
pub struct ThemeRegistry {
    themes: Vec<Theme>,
}

impl ThemeRegistry {
    /// Loads all themes: the built-in static themes (from `build.rs`), then
    /// any `*.toml` files found in `user_dir`.
    pub fn load(user_dir: Option<&Path>) -> ConfigResult<Self> {
        let themes = Theme::load_all(user_dir)?;
        Ok(Self { themes })
    }

    /// Display names of every theme, in index order.
    pub fn names(&self) -> Vec<String> {
        self.themes.iter().map(|t| t.name.clone()).collect()
    }

    /// Number of registered themes.
    pub fn len(&self) -> usize {
        self.themes.len()
    }

    /// `true` when the registry has no themes (should never happen in
    /// practice, but keeps callers honest).
    pub fn is_empty(&self) -> bool {
        self.themes.is_empty()
    }

    /// Resolves the palette at `index`, falling back to a built-in palette
    /// on out-of-range indices.
    pub fn resolve(&self, index: usize) -> Theme {
        self.themes
            .get(index)
            .cloned()
            .unwrap_or_else(Theme::fallback)
    }

    /// Returns the theme at `index`, if it exists.
    pub fn get(&self, index: usize) -> Option<&Theme> {
        self.themes.get(index)
    }

    /// Returns a copy of the registry with all colours stripped, for
    /// `NO_COLOR` terminals.
    #[must_use]
    pub fn without_colors(self) -> Self {
        Self {
            themes: self.themes.iter().map(Theme::without_colors).collect(),
        }
    }
}
