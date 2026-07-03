//! The fatal error popup.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Padding, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::layout::centered;
use crate::tui::theme::Theme;

/// Renders a centered error box with the failure message and recovery
/// hints.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, message: &str) {
    let colors = &theme.colors;
    let width = area.width.saturating_sub(8).clamp(30, 64);
    let popup = centered(area, width, 9);
    frame.render_widget(Clear, popup);

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors.danger))
        .style(Style::default().bg(colors.overlay))
        .padding(Padding::new(2, 2, 1, 1))
        .title(Line::from(Span::styled(
            " Test Failed ",
            Style::default()
                .fg(colors.danger)
                .add_modifier(Modifier::BOLD),
        )));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let lines = vec![
        Line::from(Span::styled(
            message.to_owned(),
            Style::default().fg(colors.text),
        )),
        Line::default(),
        Line::from(vec![
            Span::styled("r", Style::default().fg(colors.accent)),
            Span::styled(" retry   ", Style::default().fg(colors.subtext)),
            Span::styled("s", Style::default().fg(colors.accent)),
            Span::styled(" change server   ", Style::default().fg(colors.subtext)),
            Span::styled("q", Style::default().fg(colors.accent)),
            Span::styled(" quit", Style::default().fg(colors.subtext)),
        ]),
    ];
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}
