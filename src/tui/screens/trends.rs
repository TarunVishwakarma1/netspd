//! The result trends screen: past runs charted from history.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, Chart, Clear, Dataset, GraphType, Padding, Paragraph};
use ratatui::Frame;

use crate::app::history::HistoryRecord;
use crate::app::state::AppState;
use crate::tui::layout::centered;
use crate::tui::theme::Theme;
use crate::utils::format::format_millis;

/// Renders the trends panel: a download/upload chart across stored runs
/// with a per-server filter (←→) and summary figures.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
    let colors = &theme.colors;
    let glyphs = crate::tui::glyphs::current();
    let records = state.filtered_trends();

    let filter_label = match state.trends_filter.checked_sub(1) {
        None => "All servers".to_owned(),
        Some(index) => state
            .trend_servers()
            .get(index)
            .cloned()
            .unwrap_or_else(|| "All servers".to_owned()),
    };

    let width = area.width.saturating_sub(6).clamp(56, 100);
    let height = area.height.saturating_sub(2).clamp(16, 26);
    let popup = centered(area, width, height);
    frame.render_widget(Clear, popup);

    let block = Block::bordered()
        .border_set(glyphs.border)
        .border_style(Style::default().fg(colors.accent))
        .style(Style::default().bg(colors.overlay))
        .padding(Padding::new(2, 2, 1, 1))
        .title(Line::from(vec![
            Span::styled(
                format!(" Trends — {} runs ", records.len()),
                Style::default()
                    .fg(colors.text)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("< {filter_label} > "),
                Style::default().fg(colors.subtext),
            ),
        ]));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    if records.len() < 2 {
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

    let [chart_area, _, stats_area, ping_area] = Layout::vertical([
        Constraint::Min(8),
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(1),
    ])
    .areas(inner);

    let down: Vec<(f64, f64)> = series(&records, |r| r.download_mbps);
    let up: Vec<(f64, f64)> = series(&records, |r| r.upload_mbps);
    let max_y = down
        .iter()
        .chain(up.iter())
        .map(|(_, y)| *y)
        .fold(1.0_f64, f64::max)
        * 1.1;
    let max_x = (records.len() - 1) as f64;
    let marker = if glyphs.fancy {
        Marker::Braille
    } else {
        Marker::Dot
    };

    let datasets = vec![
        Dataset::default()
            .name("down")
            .marker(marker)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(colors.download))
            .data(&down),
        Dataset::default()
            .name("up")
            .marker(marker)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(colors.upload))
            .data(&up),
    ];
    let chart = Chart::new(datasets)
        .x_axis(
            Axis::default()
                .style(Style::default().fg(colors.muted))
                .bounds([0.0, max_x.max(1.0)])
                .labels(vec![
                    Span::styled("oldest", Style::default().fg(colors.muted)),
                    Span::styled("latest", Style::default().fg(colors.muted)),
                ]),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(colors.muted))
                .bounds([0.0, max_y])
                .labels(vec![
                    Span::styled("0", Style::default().fg(colors.muted)),
                    Span::styled(
                        format!("{:.0}", max_y / 2.0),
                        Style::default().fg(colors.muted),
                    ),
                    Span::styled(
                        format!("{max_y:.0} Mbps"),
                        Style::default().fg(colors.muted),
                    ),
                ]),
        );
    frame.render_widget(chart, chart_area);

    summary_lines(frame, stats_area, theme, &records);

    let pings: Vec<f64> = records.iter().map(|r| r.ping_ms).collect();
    let avg_ping = pings.iter().sum::<f64>() / pings.len() as f64;
    frame.render_widget(
        Paragraph::new(
            Line::from(vec![
                Span::styled("avg ping ", Style::default().fg(colors.subtext)),
                Span::styled(format_millis(avg_ping), Style::default().fg(colors.latency)),
                Span::styled(
                    "   <- -> filter by server",
                    Style::default().fg(colors.muted),
                ),
            ])
            .centered(),
        ),
        ping_area,
    );
}

/// Chart points for one metric, oldest → newest.
fn series(records: &[&HistoryRecord], metric: impl Fn(&HistoryRecord) -> f64) -> Vec<(f64, f64)> {
    records
        .iter()
        .enumerate()
        .map(|(index, record)| (index as f64, metric(record)))
        .collect()
}

/// Last/avg/best figures for both directions on two rows.
fn summary_lines(frame: &mut Frame, area: Rect, theme: &Theme, records: &[&HistoryRecord]) {
    let colors = &theme.colors;
    let glyphs = crate::tui::glyphs::current();
    let stats = |metric: fn(&HistoryRecord) -> f64| -> (f64, f64, f64) {
        let values: Vec<f64> = records.iter().map(|r| metric(r)).collect();
        let last = values.last().copied().unwrap_or(0.0);
        let avg = values.iter().sum::<f64>() / values.len().max(1) as f64;
        let best = values.iter().copied().fold(0.0_f64, f64::max);
        (last, avg, best)
    };
    let (dl, da, db) = stats(|r| r.download_mbps);
    let (ul, ua, ub) = stats(|r| r.upload_mbps);
    let lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{} ", glyphs.down),
                Style::default().fg(colors.download),
            ),
            Span::styled(
                format!("last {dl:.1}   avg {da:.1}   best {db:.1} Mbps"),
                Style::default().fg(colors.subtext),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{} ", glyphs.up),
                Style::default().fg(colors.upload),
            ),
            Span::styled(
                format!("last {ul:.1}   avg {ua:.1}   best {ub:.1} Mbps"),
                Style::default().fg(colors.subtext),
            ),
        ]),
    ];
    frame.render_widget(Paragraph::new(lines), area);
}
