//! An animated braille spinner with a label.

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::animation::spinner_frame;
use crate::tui::theme::Theme;

/// Renders a centered spinner glyph followed by a label.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, tick: u64, label: &str) {
    if area.height == 0 {
        return;
    }
    let colors = &theme.colors;
    let line = Line::from(vec![
        Span::styled(spinner_frame(tick), Style::default().fg(colors.accent)),
        Span::styled(format!(" {label}"), Style::default().fg(colors.subtext)),
    ])
    .centered();
    frame.render_widget(Paragraph::new(line), area);
}
