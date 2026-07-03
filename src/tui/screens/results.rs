//! The final results screen.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::Frame;

use crate::app::state::AppState;
use crate::tui::layout::{breakpoint, centered, Breakpoint};
use crate::tui::theme::Theme;
use crate::tui::widgets::{completion, download_card, ping_card, upload_card};

/// Renders the results screen: the completion summary with the detail
/// cards underneath on larger terminals.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
    let Some(report) = &state.report else {
        return;
    };
    let bp = breakpoint(area);

    if bp == Breakpoint::Compact || area.height < 20 {
        let summary = centered(area, area.width.min(72), 8);
        completion::render(frame, summary, theme, report);
        return;
    }

    let content = centered(area, area.width.min(96), 18);
    let [summary_area, _, cards_area] = Layout::vertical([
        Constraint::Length(8),
        Constraint::Length(2),
        Constraint::Length(7),
    ])
    .areas(content);

    completion::render(frame, summary_area, theme, report);

    let [ping_area, download_area, upload_area] = Layout::horizontal([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .spacing(2)
    .areas(cards_area);
    ping_card::render(frame, ping_area, theme, &state.ping, false);
    download_card::render(frame, download_area, theme, &state.download, false);
    upload_card::render(frame, upload_area, theme, &state.upload, false);
}
