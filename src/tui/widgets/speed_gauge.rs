//! The central speed gauge: large block digits over a live sparkline.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Sparkline};
use ratatui::Frame;

use crate::engine::metrics::Sampler;
use crate::tui::theme::Theme;
use crate::utils::format::split_bps;

use super::digits;

/// Renders the gauge: a phase label, the speed in large digits with its
/// unit, and a sparkline of recent samples underneath.
pub fn render(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    label: &str,
    bps: f64,
    color: Color,
    history: &Sampler,
) {
    if area.height == 0 {
        return;
    }
    let colors = &theme.colors;
    let (value, unit) = split_bps(bps);

    let [label_area, digits_area, spark_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(digits::FONT_HEIGHT as u16 + 1),
        Constraint::Min(0),
    ])
    .areas(area);

    frame.render_widget(
        Paragraph::new(
            Line::from(Span::styled(
                label.to_uppercase(),
                Style::default()
                    .fg(colors.subtext)
                    .add_modifier(Modifier::BOLD),
            ))
            .centered(),
        ),
        label_area,
    );

    digits::render_value(frame, digits_area, &value, unit, color, colors.muted);
    render_sparkline(frame, spark_area, history, color);
}

/// Draws the recent-sample sparkline centered under the digits.
fn render_sparkline(frame: &mut Frame, area: Rect, history: &Sampler, color: Color) {
    if area.height == 0 || area.width < 8 || history.len() < 2 {
        return;
    }
    let max = history.max().unwrap_or(0.0).max(1.0);
    let width = usize::from(area.width.saturating_sub(8));
    let data: Vec<u64> = history
        .iter()
        .map(|sample| ((sample / max) * 100.0) as u64)
        .collect();
    let take = data.len().min(width);
    let slice = &data[data.len() - take..];
    let spark_area = Rect {
        x: area.x + (area.width - take as u16) / 2,
        y: area.y,
        width: take as u16,
        height: area.height.min(2),
    };
    frame.render_widget(
        Sparkline::default()
            .data(slice)
            .max(100)
            .style(Style::default().fg(color)),
        spark_area,
    );
}
