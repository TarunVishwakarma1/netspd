//! The final results screen.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::Frame;

use crate::app::state::AppState;
use crate::tui::layout::{breakpoint, centered, Breakpoint};
use crate::tui::theme::Theme;
use crate::tui::widgets::{completion, ping_card, transfer_card};

/// Renders the results screen: the completion summary with the detail
/// cards underneath on larger terminals.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
    let Some(report) = &state.report else {
        return;
    };
    let bp = breakpoint(area);

    if bp == Breakpoint::Compact || area.height < 21 {
        let summary = centered(area, area.width.min(72), 10);
        completion::render(frame, summary, theme, report, state.speed_unit);
        return;
    }

    let content = centered(area, area.width.min(96), 19);
    let [summary_area, _, cards_area] = Layout::vertical([
        Constraint::Length(10),
        Constraint::Length(2),
        Constraint::Length(7),
    ])
    .areas(content);

    completion::render(frame, summary_area, theme, report, state.speed_unit);

    let [ping_area, download_area, upload_area] = Layout::horizontal([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .spacing(2)
    .areas(cards_area);
    ping_card::render(frame, ping_area, theme, &state.ping, false);
    transfer_card::render_download(frame, download_area, theme, &state.download, false);
    transfer_card::render_upload(frame, upload_area, theme, &state.upload, false);
}
