//! The fatal error screen.

use ratatui::layout::Rect;
use ratatui::Frame;

use crate::tui::theme::Theme;
use crate::tui::widgets::error_popup;

/// Renders the error screen (the popup centered over a quiet background).
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, message: &str) {
    error_popup::render(frame, area, theme, message);
}
