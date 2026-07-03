//! The live testing screen.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::Frame;

use crate::app::state::AppState;
use crate::engine::models::TestPhase;
use crate::tui::layout::{breakpoint, padded, Breakpoint};
use crate::tui::renderer::Motion;
use crate::tui::theme::Theme;
use crate::tui::widgets::{
    download_card, ping_card, progress_bar, speed_gauge, status_bar, upload_card,
};

/// Renders the testing screen: status row, central gauge, gradient
/// progress bar and the three metric cards.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState, motion: &Motion) {
    let colors = &theme.colors;
    let bp = breakpoint(area);
    let (gauge_height, card_height, pad) = match bp {
        Breakpoint::Compact => (7, 6, 1),
        Breakpoint::Normal => (9, 7, 2),
        Breakpoint::Wide => (11, 7, 4),
    };

    let body = padded(area, pad * 2, 0);
    let [status_area, _, gauge_area, bar_area, _, cards_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(gauge_height),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(card_height),
    ])
    .areas(body);

    let phase = state.phase;
    let (eta, ratio) = match phase {
        Some(TestPhase::Download) => (Some(state.download.eta), motion.download_ratio),
        Some(TestPhase::Upload) => (Some(state.upload.eta), motion.upload_ratio),
        _ => (None, 0.0),
    };
    status_bar::render(
        frame,
        status_area,
        theme,
        &status_bar::StatusInfo {
            phase,
            server: state.server_name(),
            tick: state.tick,
            elapsed: state.elapsed,
            eta,
        },
    );

    match phase {
        Some(TestPhase::Upload) => speed_gauge::render(
            frame,
            gauge_area,
            theme,
            "Upload",
            motion.upload_bps,
            colors.upload,
            &state.upload.history,
        ),
        Some(TestPhase::Ping) => speed_gauge::render(
            frame,
            gauge_area,
            theme,
            "Ping",
            0.0,
            colors.latency,
            &state.ping.history,
        ),
        // Download is also the resting face of the gauge before the first
        // phase event arrives.
        _ => speed_gauge::render(
            frame,
            gauge_area,
            theme,
            "Download",
            motion.download_bps,
            colors.download,
            &state.download.history,
        ),
    }

    let bar = padded(bar_area, body.width / 6, 0);
    let (from, to) = match phase {
        Some(TestPhase::Upload) => (colors.upload, colors.accent_alt),
        _ => (colors.download, colors.accent),
    };
    progress_bar::render(frame, bar, theme, ratio, from, to);

    let [ping_area, download_area, upload_area] = Layout::horizontal([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .spacing(pad)
    .areas(cards_area);

    ping_card::render(
        frame,
        ping_area,
        theme,
        &state.ping,
        phase == Some(TestPhase::Ping),
    );
    download_card::render(
        frame,
        download_area,
        theme,
        &state.download,
        phase == Some(TestPhase::Download),
    );
    upload_card::render(
        frame,
        upload_area,
        theme,
        &state.upload,
        phase == Some(TestPhase::Upload),
    );
}
