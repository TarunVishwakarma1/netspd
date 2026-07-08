//! The central speed gauge: large block digits over a live sparkline.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Sparkline};
use ratatui::Frame;

use crate::engine::metrics::Sampler;
use crate::tui::theme::Theme;
use crate::utils::format::{split_bps_unit, SpeedUnit};

use super::digits;

/// Input data for the block-digit speed gauge.
pub struct SpeedGaugeData<'a> {
    /// Phase label, e.g. `"Download"`.
    pub label: &'a str,
    /// Current speed in bits per second.
    pub bps: f64,
    /// Accent color for the digits.
    pub color: Color,
    /// Recent speed samples for the sparkline.
    pub history: &'a Sampler,
    /// Whether to display in Mbps or MB/s.
    pub speed_unit: SpeedUnit,
}

/// Renders the gauge: a phase label, the speed in large digits with its
/// unit, and a sparkline of recent samples underneath.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, data: &SpeedGaugeData<'_>) {
    if area.height == 0 {
        return;
    }
    let colors = &theme.colors;
    let (value, unit) = split_bps_unit(data.bps, data.speed_unit);

    let [label_area, digits_area, spark_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(digits::FONT_HEIGHT as u16 + 1),
        Constraint::Min(0),
    ])
    .areas(area);

    frame.render_widget(
        Paragraph::new(
            Line::from(Span::styled(
                data.label.to_uppercase(),
                Style::default()
                    .fg(colors.subtext)
                    .add_modifier(Modifier::BOLD),
            ))
            .centered(),
        ),
        label_area,
    );

    if crate::tui::glyphs::current().fancy {
        digits::render_value(frame, digits_area, &value, unit, data.color, colors.muted);
        render_sparkline(frame, spark_area, data.history, data.color);
    } else {
        // ASCII mode: plain bold text, no block digits or sparkline.
        let line = Line::from(vec![
            Span::styled(
                value,
                Style::default().fg(data.color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" {unit}"), Style::default().fg(colors.muted)),
        ])
        .centered();
        frame.render_widget(Paragraph::new(line), digits_area);
    }
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
