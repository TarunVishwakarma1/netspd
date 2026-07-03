//! The keyboard shortcut reference popup.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Padding, Paragraph};
use ratatui::Frame;

use crate::tui::layout::centered;
use crate::tui::theme::Theme;

/// The full keymap displayed in the popup.
const BINDINGS: [(&str, &str); 8] = [
    ("q / Esc", "quit"),
    ("r", "restart test"),
    ("s", "select server"),
    ("t", "select theme"),
    ("c", "view configuration"),
    ("?", "this help"),
    ("↑↓ / jk", "navigate lists"),
    ("Enter", "confirm selection"),
];

/// Renders the centered help popup over the current screen.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme) {
    let colors = &theme.colors;
    let height = BINDINGS.len() as u16 + 4;
    let popup = centered(area, 44, height);
    frame.render_widget(Clear, popup);

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors.accent))
        .style(Style::default().bg(colors.overlay))
        .padding(Padding::new(2, 2, 1, 1))
        .title(Line::from(Span::styled(
            " Keyboard ",
            Style::default()
                .fg(colors.text)
                .add_modifier(Modifier::BOLD),
        )));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let lines: Vec<Line> = BINDINGS
        .iter()
        .map(|(key, description)| {
            Line::from(vec![
                Span::styled(format!("{key:>9}  "), Style::default().fg(colors.accent)),
                Span::styled(*description, Style::default().fg(colors.subtext)),
            ])
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), inner);
}
