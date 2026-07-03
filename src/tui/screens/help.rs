//! The help screen.

use ratatui::layout::Rect;
use ratatui::Frame;

use crate::tui::theme::Theme;
use crate::tui::widgets::help_popup;

/// Renders the help screen (the popup centered over a quiet background).
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme) {
    help_popup::render(frame, area, theme);
}
