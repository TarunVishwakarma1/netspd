//! The server selection screen.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Padding, Paragraph};
use ratatui::Frame;

use crate::app::state::AppState;
use crate::tui::layout::centered;
use crate::tui::theme::Theme;

/// Maximum number of rows shown at once; the list scrolls beyond this.
const VISIBLE_ROWS: usize = 12;

/// Renders the scrollable server list with the active server marked.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
    let colors = &theme.colors;
    let width = area.width.saturating_sub(8).clamp(40, 72);
    let visible = VISIBLE_ROWS.min(state.servers.len().max(1));
    let popup = centered(area, width, visible as u16 + 4);
    frame.render_widget(Clear, popup);

    let block = Block::bordered()
        .border_set(crate::tui::glyphs::current().border)
        .border_style(Style::default().fg(colors.accent))
        .style(Style::default().bg(colors.overlay))
        .padding(Padding::new(2, 2, 1, 1))
        .title(Line::from(Span::styled(
            " Select Server ",
            Style::default()
                .fg(colors.text)
                .add_modifier(Modifier::BOLD),
        )));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    if state.servers.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "No servers available",
                Style::default().fg(colors.muted),
            ))),
            inner,
        );
        return;
    }

    // Keep the cursor visible by scrolling the window.
    let first = state
        .server_cursor
        .saturating_sub(visible.saturating_sub(1));
    let lines: Vec<Line> = state
        .servers
        .iter()
        .enumerate()
        .skip(first)
        .take(visible)
        .map(|(index, server)| {
            let is_cursor = index == state.server_cursor;
            let is_active = index == state.server_index;
            let marker = if is_cursor {
                crate::tui::glyphs::current().cursor
            } else {
                "  "
            };
            let active_mark = if is_active {
                crate::tui::glyphs::current().active
            } else {
                ""
            };
            let name_style = if is_cursor {
                Style::default()
                    .fg(colors.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors.text)
            };
            let latency_span = match server.probe_ms {
                Some(ms) => Span::styled(
                    format!("  ~{:.0}ms", ms),
                    Style::default().fg(colors.latency),
                ),
                None => Span::raw(""),
            };
            Line::from(vec![
                Span::styled(marker, Style::default().fg(colors.accent)),
                Span::styled(server.name.clone(), name_style),
                Span::styled(
                    format!("  {}", server.description),
                    Style::default().fg(colors.muted),
                ),
                latency_span,
                Span::styled(active_mark, Style::default().fg(colors.success)),
            ])
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), inner);
}
