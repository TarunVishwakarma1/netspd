//! The result trends screen: past runs charted from history.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Padding, Paragraph, Sparkline};
use ratatui::Frame;

use crate::app::history::HistoryRecord;
use crate::app::state::AppState;
use crate::tui::layout::centered;
use crate::tui::theme::Theme;
use crate::utils::format::format_millis;

/// Renders the trends panel: one sparkline per direction across stored
/// runs, with last / average / best figures.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
    let colors = &theme.colors;
    let width = area.width.saturating_sub(8).clamp(50, 84);
    let popup = centered(area, width, 16);
    frame.render_widget(Clear, popup);

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors.accent))
        .style(Style::default().bg(colors.overlay))
        .padding(Padding::new(2, 2, 1, 1))
        .title(Line::from(Span::styled(
            format!(" Trends — {} runs ", state.trends.len()),
            Style::default()
                .fg(colors.text)
                .add_modifier(Modifier::BOLD),
        )));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    if state.trends.len() < 2 {
        frame.render_widget(
            Paragraph::new(vec![
                Line::default(),
                Line::from(Span::styled(
                    "Not enough history yet — complete a few tests",
                    Style::default().fg(colors.muted),
                ))
                .centered(),
                Line::from(Span::styled(
                    "and speed trends will appear here.",
                    Style::default().fg(colors.muted),
                ))
                .centered(),
            ]),
            inner,
        );
        return;
    }

    let [download_area, _, upload_area, _, ping_area] = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(1),
        Constraint::Length(4),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);

    direction(
        frame,
        download_area,
        colors.download,
        colors.subtext,
        "↓ DOWNLOAD",
        &series(&state.trends, |record| record.download_mbps),
    );
    direction(
        frame,
        upload_area,
        colors.upload,
        colors.subtext,
        "↑ UPLOAD",
        &series(&state.trends, |record| record.upload_mbps),
    );

    let pings = series(&state.trends, |record| record.ping_ms);
    let avg_ping = pings.iter().sum::<f64>() / pings.len() as f64;
    frame.render_widget(
        Paragraph::new(
            Line::from(vec![
                Span::styled("avg ping ", Style::default().fg(colors.subtext)),
                Span::styled(format_millis(avg_ping), Style::default().fg(colors.latency)),
                Span::styled(
                    "   across all stored runs",
                    Style::default().fg(colors.muted),
                ),
            ])
            .centered(),
        ),
        ping_area,
    );
}

/// Extracts one metric from every record, oldest → newest.
fn series(records: &[HistoryRecord], metric: impl Fn(&HistoryRecord) -> f64) -> Vec<f64> {
    records.iter().map(metric).collect()
}

/// One direction: label with last/avg/best figures over a sparkline.
fn direction(
    frame: &mut Frame,
    area: Rect,
    color: Color,
    subtext: Color,
    label: &str,
    values: &[f64],
) {
    let [text_area, spark_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Length(3)]).areas(area);

    let last = values.last().copied().unwrap_or(0.0);
    let avg = values.iter().sum::<f64>() / values.len().max(1) as f64;
    let best = values.iter().copied().fold(0.0_f64, f64::max);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{label}  "),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("last {last:.1}   avg {avg:.1}   best {best:.1} Mbps"),
                Style::default().fg(subtext),
            ),
        ])),
        text_area,
    );

    let max = best.max(1.0);
    let data: Vec<u64> = values
        .iter()
        .map(|value| ((value / max) * 100.0) as u64)
        .collect();
    let take = data.len().min(usize::from(spark_area.width));
    frame.render_widget(
        Sparkline::default()
            .data(&data[data.len() - take..])
            .max(100)
            .style(Style::default().fg(color)),
        spark_area,
    );
}
