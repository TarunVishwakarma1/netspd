//! The live testing screen.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::Frame;

use crate::app::state::AppState;
use crate::engine::models::TestPhase;
use crate::tui::layout::{breakpoint, padded, Breakpoint};
use crate::tui::renderer::Motion;
use crate::tui::theme::Theme;
use crate::tui::widgets::dial_gauge::{self, DialData};
use crate::tui::widgets::{
    download_card, ping_card, progress_bar, speed_gauge, status_bar, upload_card,
};

/// Minimum gauge-slot height for the analog dial; below this the compact
/// block-digit gauge renders instead.
const DIAL_MIN_HEIGHT: u16 = 13;

/// Renders the testing screen: status row, gauge (dial or digits),
/// gradient progress bar and the three metric cards.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState, motion: &Motion) {
    let colors = &theme.colors;
    let bp = breakpoint(area);
    let (gauge_height, card_height, pad) = match bp {
        Breakpoint::Compact => (7, 6, 1),
        Breakpoint::Normal => (13, 7, 2),
        Breakpoint::Wide => (18, 7, 4),
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
    // ETA appears only once bytes move; during the ignition lead-in the
    // phase has been announced but nothing is measured yet.
    let (eta, ratio) = match phase {
        Some(TestPhase::Download) if state.download.bytes > 0 => {
            (Some(state.download.eta), motion.download_ratio)
        }
        Some(TestPhase::Upload) if state.upload.bytes > 0 => {
            (Some(state.upload.eta), motion.upload_ratio)
        }
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

    render_gauge(frame, gauge_area, theme, state, motion, bp);

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

/// Chooses and renders the central gauge for the current breakpoint:
/// twin dial cluster on Wide, single dial on Normal, block digits on
/// Compact (and always for the ping phase's gauge slot on small sizes).
fn render_gauge(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    state: &AppState,
    motion: &Motion,
    bp: Breakpoint,
) {
    let colors = &theme.colors;
    let phase = state.phase;
    let dial = area.height >= DIAL_MIN_HEIGHT && crate::tui::glyphs::current().fancy;
    let ping_ms = state
        .ping
        .stats
        .map(|stats| stats.average_ms)
        .or(state.ping.last_ms);

    if dial && bp == Breakpoint::Wide {
        // Instrument cluster: both dials side by side, active one bright.
        let [left, right] = Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .spacing(2)
            .areas(area);
        let download_active = !matches!(phase, Some(TestPhase::Upload));
        dial_gauge::render(
            frame,
            left,
            theme,
            &DialData {
                label: "Download",
                bps: motion.download_bps,
                color: colors.download,
                peak_bps: state.download.peak_bps,
                trail: &motion.download_trail,
                ping_ms,
                override_ratio: sweep_for(motion, phase, TestPhase::Download),
                dimmed: !download_active,
            },
        );
        dial_gauge::render(
            frame,
            right,
            theme,
            &DialData {
                label: "Upload",
                bps: motion.upload_bps,
                color: colors.upload,
                peak_bps: state.upload.peak_bps,
                trail: &motion.upload_trail,
                ping_ms,
                override_ratio: sweep_for(motion, phase, TestPhase::Upload),
                dimmed: download_active,
            },
        );
        return;
    }

    if dial && phase != Some(TestPhase::Ping) {
        // Single dial showing the active (or resting download) phase.
        let upload = phase == Some(TestPhase::Upload);
        let (label, bps, color, peak, trail, active_phase) = if upload {
            (
                "Upload",
                motion.upload_bps,
                colors.upload,
                state.upload.peak_bps,
                motion.upload_trail.as_slice(),
                TestPhase::Upload,
            )
        } else {
            (
                "Download",
                motion.download_bps,
                colors.download,
                state.download.peak_bps,
                motion.download_trail.as_slice(),
                TestPhase::Download,
            )
        };
        dial_gauge::render(
            frame,
            area,
            theme,
            &DialData {
                label,
                bps,
                color,
                peak_bps: peak,
                trail,
                ping_ms,
                override_ratio: sweep_for(motion, phase, active_phase),
                dimmed: false,
            },
        );
        return;
    }

    // Compact terminals and the ping phase keep the block-digit gauge.
    match phase {
        Some(TestPhase::Upload) => speed_gauge::render(
            frame,
            area,
            theme,
            "Upload",
            motion.upload_bps,
            colors.upload,
            &state.upload.history,
        ),
        Some(TestPhase::Ping) => speed_gauge::render(
            frame,
            area,
            theme,
            "Ping",
            0.0,
            colors.latency,
            &state.ping.history,
        ),
        _ => speed_gauge::render(
            frame,
            area,
            theme,
            "Download",
            motion.download_bps,
            colors.download,
            &state.download.history,
        ),
    }
}

/// The ignition sweep override, applied only to the dial whose phase is
/// currently running.
fn sweep_for(motion: &Motion, current: Option<TestPhase>, dial_phase: TestPhase) -> Option<f64> {
    if current == Some(dial_phase) {
        motion.sweep_ratio
    } else {
        None
    }
}
