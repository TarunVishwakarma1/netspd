//! The download metrics card.

use ratatui::layout::Rect;
use ratatui::Frame;

use crate::app::state::TransferView;
use crate::tui::theme::Theme;

use super::transfer_card::{self, CardStyle};

/// Renders the download card.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, view: &TransferView, active: bool) {
    let style = CardStyle {
        title: "Download",
        icon: "↓",
        color: theme.colors.download,
    };
    transfer_card::render(frame, area, theme, style, view, active);
}
