//! The final results screen.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::state::AppState;
use crate::tui::layout::{breakpoint, centered, Breakpoint};
use crate::tui::theme::Theme;
use crate::tui::widgets::{completion, download_card, ping_card, upload_card};
use crate::utils::format::format_duration;

/// Renders the results screen: the completion summary with the detail
/// cards underneath on larger terminals.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
    let Some(report) = &state.report else {
        return;
    };
    let bp = breakpoint(area);

    if bp == Breakpoint::Compact || area.height < 20 {
        let summary = centered(area, area.width.min(72), 9);
        completion::render(frame, summary, theme, report);
        render_repeat_countdown(frame, area, theme, state);
        return;
    }

    let content = centered(area, area.width.min(96), 18);
    let [summary_area, _, cards_area] = Layout::vertical([
        Constraint::Length(9),
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
    render_repeat_countdown(frame, area, theme, state);
}

/// A quiet status row at the bottom: transient notices take priority
/// over the auto-repeat countdown.
fn render_repeat_countdown(frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
    if area.height < 2 {
        return;
    }
    let colors = &theme.colors;
    let row = Rect {
        y: area.y + area.height - 1,
        height: 1,
        ..area
    };

    if let Some(notice) = state.notice() {
        let line = Line::from(Span::styled(
            notice.to_owned(),
            Style::default().fg(colors.success),
        ))
        .centered();
        frame.render_widget(Paragraph::new(line), row);
        return;
    }

    let Some(remaining) = state.repeat_remaining() else {
        return;
    };
    let line = Line::from(vec![
        Span::styled(
            format!("{} next test in ", crate::tui::glyphs::current().repeat),
            Style::default().fg(colors.muted),
        ),
        Span::styled(
            format_duration(remaining),
            Style::default().fg(colors.subtext),
        ),
    ])
    .centered();
    frame.render_widget(Paragraph::new(line), row);
}
