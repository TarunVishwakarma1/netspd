//! The live status row shown while a test runs.

use std::time::Duration;

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::engine::models::TestPhase;
use crate::tui::animation::spinner_frame;
use crate::tui::theme::Theme;
use crate::utils::format::{format_duration, format_eta};

/// Everything the status bar displays.
#[derive(Debug, Clone, Copy)]
pub struct StatusInfo<'a> {
    /// The running phase, or `None` when the test is complete.
    pub phase: Option<TestPhase>,
    /// Name of the server being tested.
    pub server: &'a str,
    /// UI tick, drives the spinner.
    pub tick: u64,
    /// Time elapsed since the test started.
    pub elapsed: Duration,
    /// Estimated time remaining in the current phase.
    pub eta: Option<Duration>,
}

/// Renders phase, server and timing information on a single row.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, info: &StatusInfo) {
    if area.height == 0 {
        return;
    }
    let colors = &theme.colors;

    let left = match info.phase {
        Some(phase) => Line::from(vec![
            Span::styled(
                format!("  {} ", spinner_frame(info.tick)),
                Style::default().fg(colors.accent),
            ),
            Span::styled(
                phase.label(),
                Style::default()
                    .fg(colors.text)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        None => Line::from(Span::styled(
            "  ✓ Complete",
            Style::default().fg(colors.success),
        )),
    };
    frame.render_widget(Paragraph::new(left), area);

    frame.render_widget(
        Paragraph::new(
            Line::from(Span::styled(
                info.server.to_owned(),
                Style::default().fg(colors.subtext),
            ))
            .centered(),
        ),
        area,
    );

    let mut right = vec![Span::styled(
        format_duration(info.elapsed),
        Style::default().fg(colors.subtext),
    )];
    if let Some(eta) = info.eta {
        right.push(Span::styled("  eta ", Style::default().fg(colors.muted)));
        right.push(Span::styled(
            format_eta(eta),
            Style::default().fg(colors.subtext),
        ));
    }
    right.push(Span::raw("  "));
    frame.render_widget(Paragraph::new(Line::from(right).right_aligned()), area);
}
