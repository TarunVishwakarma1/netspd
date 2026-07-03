//! The bottom key-hint footer.

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::theme::Theme;

/// A single `key → label` hint.
pub type Hint = (&'static str, &'static str);

/// Renders a centered row of key hints, e.g. `q quit · r restart`.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, hints: &[Hint]) {
    if area.height == 0 {
        return;
    }
    let colors = &theme.colors;
    let mut spans = Vec::with_capacity(hints.len() * 3);
    for (index, (key, label)) in hints.iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled("  ·  ", Style::default().fg(colors.muted)));
        }
        spans.push(Span::styled(*key, Style::default().fg(colors.accent)));
        spans.push(Span::styled(
            format!(" {label}"),
            Style::default().fg(colors.subtext),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(spans).centered()), area);
}
