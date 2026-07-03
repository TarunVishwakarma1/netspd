//! The startup splash screen.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::layout::centered;
use crate::tui::theme::Theme;
use crate::tui::widgets::spinner;

/// The block-letter wordmark.
const LOGO: [&str; 3] = [
    "█▀█ █▀▀ ▀█▀ █▀▀ █▀█ █▀▄",
    "█ █ █▀▀  █  ▀▀█ █▀▀ █ █",
    "▀ ▀ ▀▀▀  ▀  ▀▀▀ ▀   ▀▀ ",
];

/// Renders the splash: wordmark, tagline and a discovery spinner.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, tick: u64, status: &str) {
    let colors = &theme.colors;
    let content = centered(area, 40, 8);

    let [logo_area, tagline_area, _, spinner_area] = Layout::vertical([
        Constraint::Length(LOGO.len() as u16 + 1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(content);

    let logo_lines: Vec<Line> = LOGO
        .iter()
        .map(|row| Line::from(Span::styled(*row, Style::default().fg(colors.accent))).centered())
        .collect();
    frame.render_widget(Paragraph::new(logo_lines), logo_area);

    frame.render_widget(
        Paragraph::new(
            Line::from(Span::styled(
                "network speed, beautifully measured",
                Style::default().fg(colors.muted),
            ))
            .centered(),
        ),
        tagline_area,
    );

    spinner::render(frame, spinner_area, theme, tick, status);
}
