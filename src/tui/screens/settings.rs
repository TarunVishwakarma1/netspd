//! The read-only configuration screen.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Padding, Paragraph};
use ratatui::Frame;

use crate::app::state::AppState;
use crate::tui::layout::centered;
use crate::tui::theme::Theme;

/// Renders the active configuration as a centered card.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
    let colors = &theme.colors;
    let settings = &state.settings;
    let engine = &settings.engine;

    let entries: [(&str, String); 9] = [
        (
            "theme",
            state
                .theme_names
                .get(state.theme_index)
                .cloned()
                .unwrap_or_default(),
        ),
        ("provider", state.provider_name.to_owned()),
        ("refresh rate", format!("{} fps", settings.refresh_rate)),
        (
            "animation speed",
            format!("{:.1}×", settings.animation_speed()),
        ),
        ("ping samples", engine.ping_samples.to_string()),
        ("phase duration", format!("{} s", engine.duration_secs)),
        ("connections", engine.connections.to_string()),
        ("timeout", format!("{} s", engine.timeout_secs)),
        ("upload chunk", format!("{} KB", engine.upload_chunk_kb)),
    ];

    let popup = centered(area, 46, entries.len() as u16 + 4);
    frame.render_widget(Clear, popup);

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors.accent))
        .style(Style::default().bg(colors.overlay))
        .padding(Padding::new(2, 2, 1, 1))
        .title(Line::from(Span::styled(
            " Configuration ",
            Style::default()
                .fg(colors.text)
                .add_modifier(Modifier::BOLD),
        )));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let lines: Vec<Line> = entries
        .iter()
        .map(|(key, value)| {
            Line::from(vec![
                Span::styled(format!("{key:<16}"), Style::default().fg(colors.subtext)),
                Span::styled(value.clone(), Style::default().fg(colors.text)),
            ])
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), inner);
}
