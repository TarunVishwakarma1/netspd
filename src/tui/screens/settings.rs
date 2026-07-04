//! The interactive configuration screen.
//!
//! Rows are selected with ↑↓ and adjusted in place with ←→ (within the
//! same clamps the loader enforces); `w` writes the result back to the
//! user's `config.toml`. Theme changes apply live; engine values apply
//! on the next test.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Padding, Paragraph};
use ratatui::Frame;

use crate::app::state::AppState;
use crate::tui::layout::centered;
use crate::tui::theme::Theme;

/// Renders the editable configuration card.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
    let colors = &theme.colors;
    let glyphs = crate::tui::glyphs::current();
    let settings = &state.settings;
    let engine = &settings.engine;

    let rows: [(&str, String); AppState::SETTINGS_ROWS] = [
        (
            "theme",
            state
                .theme_names
                .get(state.theme_index)
                .cloned()
                .unwrap_or_default(),
        ),
        ("refresh rate", format!("{} fps", settings.refresh_rate)),
        (
            "animation speed",
            format!("{:.1}x", settings.animation_speed()),
        ),
        ("ping samples", engine.ping_samples.to_string()),
        ("phase duration", format!("{} s", engine.duration_secs)),
        ("connections", engine.connections.to_string()),
        ("timeout", format!("{} s", engine.timeout_secs)),
        ("upload chunk", format!("{} KB", engine.upload_chunk_kb)),
    ];

    let popup = centered(area, 52, rows.len() as u16 + 8);
    frame.render_widget(Clear, popup);

    let block = Block::bordered()
        .border_set(glyphs.border)
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

    let mut lines: Vec<Line> = Vec::with_capacity(rows.len() + 4);
    for (index, (key, value)) in rows.iter().enumerate() {
        let selected = index == state.settings_cursor;
        let marker = if selected { glyphs.cursor } else { "  " };
        let key_style = if selected {
            Style::default()
                .fg(colors.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(colors.subtext)
        };
        let value_text = if selected {
            format!("< {value} >")
        } else {
            value.clone()
        };
        lines.push(Line::from(vec![
            Span::styled(marker, Style::default().fg(colors.accent)),
            Span::styled(format!("{key:<16}"), key_style),
            Span::styled(value_text, Style::default().fg(colors.text)),
        ]));
    }
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        format!("  provider        {} (via --provider)", state.provider_name),
        Style::default().fg(colors.muted),
    )));
    lines.push(Line::default());
    if let Some(notice) = state.notice() {
        lines.push(Line::from(Span::styled(
            format!("  {notice}"),
            Style::default().fg(colors.success),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "  w saves; engine values apply on the next test",
            Style::default().fg(colors.muted),
        )));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}
