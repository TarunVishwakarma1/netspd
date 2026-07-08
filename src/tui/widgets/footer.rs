//! The bottom key-hint footer and per-screen hint tables.

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::screen::Screen;
use crate::app::state::AppState;
use crate::tui::theme::Theme;
use crate::utils::format::format_duration;

/// A single `key → label` hint.
pub type Hint = (&'static str, &'static str);

/// Renders the footer for one frame: notice/countdown override on the
/// Results screen, otherwise the static key-hint bar for the active screen.
pub fn render_frame(frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
    if area.height == 0 {
        return;
    }
    let hint_row = Rect {
        y: area.y + area.height.saturating_sub(1),
        height: 1,
        ..area
    };
    let colors = &theme.colors;

    if state.screen == Screen::Results {
        if let Some(notice) = state.notice() {
            let line = Line::from(Span::styled(
                notice.to_owned(),
                Style::default().fg(colors.success),
            ))
            .centered();
            frame.render_widget(Paragraph::new(line), hint_row);
            return;
        }
        if let Some(remaining) = state.repeat_remaining() {
            let line = Line::from(vec![
                Span::styled(
                    format!("{} next test in ", crate::tui::glyphs::current().repeat),
                    Style::default().fg(colors.muted),
                ),
                Span::styled(
                    format_duration(remaining),
                    Style::default().fg(colors.subtext),
                ),
            ])
            .centered();
            frame.render_widget(Paragraph::new(line), hint_row);
            return;
        }
    }

    render(frame, hint_row, theme, hints_for(state.screen));
}

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

/// Key hints for each screen.
fn hints_for(screen: Screen) -> &'static [Hint] {
    match screen {
        Screen::Splash => &[("q", "quit"), ("?", "help")],
        Screen::Testing => &[
            ("q", "quit"),
            ("r", "restart"),
            ("s", "servers"),
            ("t", "theme"),
            ("?", "help"),
        ],
        Screen::Results => &[
            ("r", "run again"),
            ("u", "unit"),
            ("y", "copy"),
            ("g", "trends"),
            ("s", "servers"),
            ("c", "config"),
            ("?", "help"),
        ],
        Screen::Help => &[("Esc", "back"), ("q", "quit")],
        Screen::Settings => &[
            ("↑↓", "select"),
            ("←→", "adjust"),
            ("w", "save"),
            ("Esc", "back"),
        ],
        Screen::Trends => &[("←→", "filter server"), ("Esc", "back"), ("q", "quit")],
        Screen::ServerSelect | Screen::ThemeSelect => {
            &[("↑↓", "navigate"), ("Enter", "select"), ("Esc", "back")]
        }
        Screen::Error => &[("r", "retry"), ("s", "servers"), ("q", "quit")],
    }
}
