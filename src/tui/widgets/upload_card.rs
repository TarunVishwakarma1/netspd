//! The upload metrics card.

use ratatui::layout::Rect;
use ratatui::Frame;

use crate::app::state::TransferView;
use crate::tui::theme::Theme;

use super::transfer_card::{self, CardStyle};

/// Renders the upload card.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, view: &TransferView, active: bool) {
    let style = CardStyle {
        title: "Upload",
        icon: "↑",
        color: theme.colors.upload,
    };
    transfer_card::render(frame, area, theme, style, view, active);
}
