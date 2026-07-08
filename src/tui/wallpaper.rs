//! Background wallpaper rendering.
//!
//! Replaces the solid background block in the renderer with the wallpaper
//! configured under `[wallpaper]` in `config.toml`.

use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::Frame;

use crate::config::{WallpaperKind, WallpaperSection};
use crate::tui::theme::{blend, parse_hex, Colors};

/// Renders the configured wallpaper into `area` as the first layer.
///
/// Called before any UI widgets so they float on top. Each widget still
/// paints its own surface/overlay background on its cells; the wallpaper
/// shows through wherever a widget does not set a background.
pub fn render(frame: &mut Frame, area: Rect, colors: &Colors, wallpaper: &WallpaperSection) {
    match wallpaper.kind {
        WallpaperKind::None => {
            frame.render_widget(
                Block::default().style(Style::default().bg(colors.background)),
                area,
            );
        }
        WallpaperKind::Gradient => {
            let from = resolve_color(wallpaper.from.as_deref(), colors.background);
            let to = wallpaper
                .to
                .as_deref()
                .and_then(|s| parse_hex(s).ok())
                .unwrap_or_else(|| blend(colors.background, Color::Rgb(0, 0, 0), 0.5));

            render_gradient(frame, area, from, to);
        }
    }
}

fn resolve_color(hex: Option<&str>, fallback: Color) -> Color {
    hex.and_then(|s| parse_hex(s).ok()).unwrap_or(fallback)
}

/// Renders a static vertical gradient, one `Block` per row.
fn render_gradient(frame: &mut Frame, area: Rect, from: Color, to: Color) {
    let height = area.height;
    if height == 0 {
        return;
    }
    for y in area.top()..area.bottom() {
        let blend_t = if height > 1 {
            f64::from(y - area.top()) / f64::from(height - 1)
        } else {
            0.0
        };
        let row_color = blend(from, to, blend_t);
        let row_rect = Rect::new(area.x, y, area.width, 1);
        frame.render_widget(
            Block::default().style(Style::default().bg(row_color)),
            row_rect,
        );
    }
}
