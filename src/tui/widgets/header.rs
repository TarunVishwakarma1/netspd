//! The top application header.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::theme::Theme;

/// Renders the header: brand on the left, context on the right, separated
/// from the body by generous whitespace instead of borders.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, context: &str) {
    if area.height == 0 {
        return;
    }
    let colors = &theme.colors;
    let brand = Line::from(vec![
        Span::styled(
            "  netspd",
            Style::default()
                .fg(colors.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  v{}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(colors.muted),
        ),
    ]);
    frame.render_widget(Paragraph::new(brand), area);

    let context_line = Line::from(Span::styled(
        format!("{context}  "),
        Style::default().fg(colors.subtext),
    ))
    .right_aligned();
    frame.render_widget(Paragraph::new(context_line), area);
}
