//! The central speed gauge: large block digits over a live sparkline.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Sparkline};
use ratatui::Frame;

use crate::engine::metrics::Sampler;
use crate::tui::theme::Theme;
use crate::utils::format::split_bps;

/// Height of the block digit font.
const FONT_HEIGHT: usize = 3;

/// 4-column block glyphs for digits `0-9`; index 10 is the decimal point.
const GLYPHS: [[&str; FONT_HEIGHT]; 11] = [
    ["▄▀▀▄", "█  █", "▀▄▄▀"],
    [" ▄█ ", "  █ ", " ▄█▄"],
    ["▄▀▀▄", " ▄▄▀", "▄█▄▄"],
    ["▄▀▀▄", " ▀▀▄", "▀▄▄▀"],
    ["▄  ▄", "█▄▄█", "   █"],
    ["█▀▀▀", "▀▀▀▄", "▄▄▄▀"],
    ["▄▀▀ ", "█▀▀▄", "▀▄▄▀"],
    ["▀▀▀█", "  █ ", " █  "],
    ["▄▀▀▄", "▄▀▀▄", "▀▄▄▀"],
    ["▄▀▀▄", "▀▄▄█", " ▄▄▀"],
    ["  ", "  ", "▄ "],
];

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
        Constraint::Length(FONT_HEIGHT as u16 + 1),
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

    render_digits(frame, digits_area, &value, unit, color, colors.muted);
    render_sparkline(frame, spark_area, history, color);
}

/// Draws the numeric value in block digits, falling back to plain bold
/// text when the area is too small for the font.
fn render_digits(
    frame: &mut Frame,
    area: Rect,
    value: &str,
    unit: &str,
    color: Color,
    unit_color: Color,
) {
    if area.height == 0 {
        return;
    }
    let rows = big_rows(value);
    let digits_width = rows[0].chars().count() as u16;
    let total_width = digits_width + unit.len() as u16 + 1;

    if area.height < FONT_HEIGHT as u16 || area.width < total_width {
        let line = Line::from(vec![
            Span::styled(
                value.to_owned(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" {unit}"), Style::default().fg(unit_color)),
        ])
        .centered();
        frame.render_widget(Paragraph::new(line), area);
        return;
    }

    let x = area.x + (area.width - total_width) / 2;
    for (row_index, row) in rows.iter().enumerate() {
        let row_area = Rect {
            x,
            y: area.y + row_index as u16,
            width: digits_width,
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                row.clone(),
                Style::default().fg(color),
            ))),
            row_area,
        );
    }
    // Unit sits on the baseline row, just after the digits.
    let unit_area = Rect {
        x: x + digits_width + 1,
        y: area.y + FONT_HEIGHT as u16 - 1,
        width: unit.len() as u16,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            unit.to_owned(),
            Style::default().fg(unit_color),
        ))),
        unit_area,
    );
}

/// Builds the three text rows of the block-digit rendering of `value`.
fn big_rows(value: &str) -> [String; FONT_HEIGHT] {
    let mut rows = [String::new(), String::new(), String::new()];
    for ch in value.chars() {
        let glyph = match ch {
            '0'..='9' => &GLYPHS[(ch as usize) - ('0' as usize)],
            '.' => &GLYPHS[10],
            _ => continue,
        };
        for (row, part) in rows.iter_mut().zip(glyph.iter()) {
            row.push_str(part);
            row.push(' ');
        }
    }
    rows
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
