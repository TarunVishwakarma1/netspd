//! The latency metrics card.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Paragraph};
use ratatui::Frame;

use crate::app::state::PingView;
use crate::tui::theme::Theme;
use crate::utils::format::format_millis;

/// Renders the ping card: headline latency plus jitter, range and loss.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, view: &PingView, active: bool) {
    if area.height == 0 {
        return;
    }
    let colors = &theme.colors;
    let border_color = if active {
        colors.latency
    } else {
        colors.border
    };
    let glyphs = crate::tui::glyphs::current();
    let block = Block::bordered()
        .border_set(glyphs.border)
        .border_style(Style::default().fg(border_color))
        .padding(Padding::horizontal(1))
        .title(Line::from(vec![
            Span::styled(
                format!(" {} ", glyphs.latency),
                Style::default().fg(colors.latency),
            ),
            Span::styled(
                "Ping ",
                Style::default()
                    .fg(colors.text)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let headline = view
        .stats
        .map(|stats| format_millis(stats.average_ms))
        .or_else(|| view.last_ms.map(format_millis))
        .unwrap_or_else(|| "—".to_owned());

    let (jitter, range, loss) = match view.stats {
        Some(stats) => (
            format_millis(stats.jitter_ms),
            format!(
                "{} – {}",
                format_millis(stats.min_ms),
                format_millis(stats.max_ms)
            ),
            format!("{:.0}%", stats.packet_loss_pct),
        ),
        None => ("—".to_owned(), "—".to_owned(), "—".to_owned()),
    };

    let label = Style::default().fg(colors.subtext);
    let value = Style::default().fg(colors.text);
    let lines = vec![
        Line::from(Span::styled(
            headline,
            Style::default()
                .fg(colors.latency)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("jitter  ", label),
            Span::styled(jitter, value),
        ]),
        Line::from(vec![
            Span::styled("range   ", label),
            Span::styled(range, value),
        ]),
        Line::from(vec![
            Span::styled("loss    ", label),
            Span::styled(loss, value),
        ]),
    ];
    frame.render_widget(Paragraph::new(lines), inner);
}
