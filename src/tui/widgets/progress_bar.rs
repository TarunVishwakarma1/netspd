//! A smooth, gradient progress bar.
//!
//! Uses eighth-block characters for a soft leading edge, or plain `#`
//! fills in ASCII mode.

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::glyphs;
use crate::tui::theme::{blend, Theme};

/// Renders a single-row progress bar.
///
/// The filled portion sweeps a color gradient from `from` to `to`; the
/// empty portion is a quiet muted track.
pub fn render(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    ratio: f64,
    from: ratatui::style::Color,
    to: ratatui::style::Color,
) {
    if area.height == 0 || area.width == 0 {
        return;
    }
    let colors = &theme.colors;
    let glyphs = glyphs::current();
    let width = area.width as usize;
    let ratio = ratio.clamp(0.0, 1.0);
    let cells = ratio * width as f64;
    let full = cells.floor() as usize;
    let remainder = cells - full as f64;

    let mut spans = Vec::with_capacity(width);
    for index in 0..full.min(width) {
        let t = if width > 1 {
            index as f64 / (width - 1) as f64
        } else {
            0.0
        };
        spans.push(Span::styled(
            glyphs.bar_fill,
            Style::default().fg(blend(from, to, t)),
        ));
    }
    if full < width && remainder > 0.0 {
        let partials = glyphs.bar_partials;
        let partial_index =
            ((remainder * partials.len() as f64).floor() as usize).min(partials.len() - 1);
        let t = full as f64 / (width.max(2) - 1) as f64;
        spans.push(Span::styled(
            partials[partial_index],
            Style::default().fg(blend(from, to, t)),
        ));
    }
    let used = spans.len();
    if used < width {
        spans.push(Span::styled(
            glyphs.bar_track.repeat(width - used),
            Style::default().fg(colors.border),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
