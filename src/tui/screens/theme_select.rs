//! The theme selection screen with live palette swatches.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Padding, Paragraph};
use ratatui::Frame;

use crate::app::state::AppState;
use crate::tui::layout::centered;
use crate::tui::theme::Theme;

/// Renders the theme list; each row previews the theme's key colors.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState, themes: &[Theme]) {
    let colors = &theme.colors;
    let popup = centered(area, 44, state.theme_names.len() as u16 + 4);
    frame.render_widget(Clear, popup);

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors.accent))
        .style(Style::default().bg(colors.overlay))
        .padding(Padding::new(2, 2, 1, 1))
        .title(Line::from(Span::styled(
            " Select Theme ",
            Style::default()
                .fg(colors.text)
                .add_modifier(Modifier::BOLD),
        )));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let lines: Vec<Line> = state
        .theme_names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let is_cursor = index == state.theme_cursor;
            let is_active = index == state.theme_index;
            let marker = if is_cursor { "▸ " } else { "  " };
            let active_mark = if is_active { " ●" } else { "" };
            let name_style = if is_cursor {
                Style::default()
                    .fg(colors.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors.text)
            };
            let mut spans = vec![
                Span::styled(marker, Style::default().fg(colors.accent)),
                Span::styled(format!("{name:<12}"), name_style),
            ];
            if let Some(preview) = themes.get(index) {
                let palette = [
                    preview.colors.accent,
                    preview.colors.download,
                    preview.colors.upload,
                    preview.colors.success,
                    preview.colors.warning,
                ];
                for swatch in palette {
                    spans.push(Span::styled("██", Style::default().fg(swatch)));
                }
            }
            spans.push(Span::styled(
                active_mark,
                Style::default().fg(colors.success),
            ));
            Line::from(spans)
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), inner);
}
