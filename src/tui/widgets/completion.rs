//! The results summary shown when a test completes.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::engine::models::{TestReport, TransferStats};
use crate::tui::theme::Theme;
use crate::utils::format::{format_bytes, format_millis, split_bps};

/// Renders the completion summary: download and upload headline numbers
/// side by side, with latency details and totals underneath.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, report: &TestReport) {
    if area.height < 6 {
        return;
    }
    let colors = &theme.colors;

    let [title_area, _, speeds_area, _, latency_area, verdict_area, totals_area] =
        Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(area);

    frame.render_widget(
        Paragraph::new(
            Line::from(Span::styled(
                format!("{} TEST COMPLETE", crate::tui::glyphs::current().check),
                Style::default()
                    .fg(colors.success)
                    .add_modifier(Modifier::BOLD),
            ))
            .centered(),
        ),
        title_area,
    );

    let [download_area, upload_area] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(speeds_area);
    headline(
        frame,
        download_area,
        &format!("{} DOWNLOAD", crate::tui::glyphs::current().down),
        &report.download,
        colors.download,
        colors.subtext,
    );
    headline(
        frame,
        upload_area,
        &format!("{} UPLOAD", crate::tui::glyphs::current().up),
        &report.upload,
        colors.upload,
        colors.subtext,
    );

    let latency = &report.latency;
    let mut latency_spans = vec![
        Span::styled("ping ", Style::default().fg(colors.subtext)),
        Span::styled(
            format_millis(latency.average_ms),
            Style::default().fg(colors.latency),
        ),
        Span::styled("   jitter ", Style::default().fg(colors.subtext)),
        Span::styled(
            format_millis(latency.jitter_ms),
            Style::default().fg(colors.latency),
        ),
        Span::styled("   loss ", Style::default().fg(colors.subtext)),
        Span::styled(
            format!("{:.0}%", latency.packet_loss_pct),
            Style::default().fg(colors.latency),
        ),
    ];
    if let Some(bloat) = report.bufferbloat {
        let grade_color = match bloat.grade {
            crate::engine::models::BufferbloatGrade::APlus
            | crate::engine::models::BufferbloatGrade::A => colors.success,
            crate::engine::models::BufferbloatGrade::B
            | crate::engine::models::BufferbloatGrade::C => colors.warning,
            _ => colors.danger,
        };
        latency_spans.push(Span::styled(
            "   bufferbloat ",
            Style::default().fg(colors.subtext),
        ));
        latency_spans.push(Span::styled(
            bloat.grade.label(),
            Style::default()
                .fg(grade_color)
                .add_modifier(Modifier::BOLD),
        ));
    }
    frame.render_widget(
        Paragraph::new(Line::from(latency_spans).centered()),
        latency_area,
    );

    frame.render_widget(
        Paragraph::new(
            Line::from(Span::styled(
                crate::app::verdict::verdict(report),
                Style::default().fg(colors.subtext),
            ))
            .centered(),
        ),
        verdict_area,
    );

    let transferred = report.download.bytes + report.upload.bytes;
    frame.render_widget(
        Paragraph::new(
            Line::from(vec![
                Span::styled(
                    format!("{}   ", report.server_name),
                    Style::default().fg(colors.muted),
                ),
                Span::styled(
                    format!("{} transferred", format_bytes(transferred)),
                    Style::default().fg(colors.muted),
                ),
            ])
            .centered(),
        ),
        totals_area,
    );
}

/// One large speed figure with its label and peak value.
fn headline(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    stats: &TransferStats,
    color: Color,
    subtext: Color,
) {
    let (value, unit) = split_bps(stats.average_bps);
    let (peak, peak_unit) = split_bps(stats.peak_bps);
    let lines = vec![
        Line::from(Span::styled(label.to_owned(), Style::default().fg(subtext))).centered(),
        Line::from(vec![
            Span::styled(
                value,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" {unit}"), Style::default().fg(subtext)),
        ])
        .centered(),
        Line::from(Span::styled(
            format!("peak {peak} {peak_unit}"),
            Style::default().fg(subtext),
        ))
        .centered(),
    ];
    frame.render_widget(Paragraph::new(lines), area);
}
