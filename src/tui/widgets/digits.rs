//! The shared block-digit font used for large numeric readouts.

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

/// Height of the block digit font, in rows.
pub const FONT_HEIGHT: usize = 3;

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

/// Builds the three text rows of the block-digit rendering of `value`.
///
/// Characters other than digits and `.` are skipped.
#[must_use]
pub fn big_rows(value: &str) -> [String; FONT_HEIGHT] {
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

/// Draws a numeric value in block digits with its unit on the baseline,
/// falling back to plain bold text when the area is too small for the
/// font.
pub fn render_value(
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
