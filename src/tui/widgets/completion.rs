//! The results summary shown when a test completes.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::score::CompositeScore;
use crate::engine::models::{TestReport, TransferStats};
use crate::tui::theme::{Colors, Theme};
use crate::utils::format::{format_bytes, format_millis, split_bps_unit, SpeedUnit};

/// Renders the completion summary: title, speeds, latency row, score,
/// verdict and totals — each in its own named sub-function.
pub fn render(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    report: &TestReport,
    speed_unit: SpeedUnit,
) {
    if area.height < 7 {
        return;
    }
    let colors = &theme.colors;

    let [title_area, _, speeds_area, _, latency_area, score_area, verdict_area, totals_area] =
        Layout::vertical([
            Constraint::Length(1), // title
            Constraint::Length(1), // gap
            Constraint::Length(3), // download + upload side by side
            Constraint::Length(1), // gap
            Constraint::Length(1), // ping / jitter / loss / bufferbloat
            Constraint::Length(1), // composite score
            Constraint::Length(1), // plain-language verdict
            Constraint::Length(1), // server name + bytes + ip version
        ])
        .areas(area);

    render_title(frame, title_area, colors);
    render_speeds(frame, speeds_area, colors, report, speed_unit);
    render_latency_row(frame, latency_area, colors, report);
    render_score_row(frame, score_area, colors, report);
    render_verdict_row(frame, verdict_area, colors, report);
    render_totals_row(frame, totals_area, colors, report);
}

// ── Section renderers ────────────────────────────────────────────────────────

fn render_title(frame: &mut Frame, area: Rect, colors: &Colors) {
    let line = Line::from(Span::styled(
        format!("{} TEST COMPLETE", crate::tui::glyphs::current().check),
        Style::default()
            .fg(colors.success)
            .add_modifier(Modifier::BOLD),
    ))
    .centered();
    frame.render_widget(Paragraph::new(line), area);
}

fn render_speeds(
    frame: &mut Frame,
    area: Rect,
    colors: &Colors,
    report: &TestReport,
    speed_unit: SpeedUnit,
) {
    let [download_area, upload_area] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);

    render_speed_headline(
        frame,
        download_area,
        &format!("{} DOWNLOAD", crate::tui::glyphs::current().down),
        &report.download,
        colors.speed_color(report.download.average_bps),
        colors.subtext,
        speed_unit,
    );
    render_speed_headline(
        frame,
        upload_area,
        &format!("{} UPLOAD", crate::tui::glyphs::current().up),
        &report.upload,
        colors.speed_color(report.upload.average_bps),
        colors.subtext,
        speed_unit,
    );
}

fn render_latency_row(frame: &mut Frame, area: Rect, colors: &Colors, report: &TestReport) {
    let latency = &report.latency;
    let mut spans = vec![
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
        spans.push(Span::styled(
            "   bufferbloat ",
            Style::default().fg(colors.subtext),
        ));
        spans.push(Span::styled(
            bloat.grade.label(),
            Style::default()
                .fg(bloat.grade.color(colors))
                .add_modifier(Modifier::BOLD),
        ));
    }

    frame.render_widget(Paragraph::new(Line::from(spans).centered()), area);
}

fn render_score_row(frame: &mut Frame, area: Rect, colors: &Colors, report: &TestReport) {
    let score = CompositeScore::compute(report);
    let grade_color = score.grade.color(colors);
    let line = Line::from(vec![
        Span::styled("score ", Style::default().fg(colors.subtext)),
        Span::styled(
            score.points.to_string(),
            Style::default()
                .fg(grade_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("/100 ", Style::default().fg(colors.muted)),
        Span::styled(
            score.grade.label(),
            Style::default()
                .fg(grade_color)
                .add_modifier(Modifier::BOLD),
        ),
    ])
    .centered();
    frame.render_widget(Paragraph::new(line), area);
}

fn render_verdict_row(frame: &mut Frame, area: Rect, colors: &Colors, report: &TestReport) {
    let line = Line::from(Span::styled(
        crate::app::verdict::verdict(report),
        Style::default().fg(colors.subtext),
    ))
    .centered();
    frame.render_widget(Paragraph::new(line), area);
}

fn render_totals_row(frame: &mut Frame, area: Rect, colors: &Colors, report: &TestReport) {
    let transferred = report.download.bytes + report.upload.bytes;
    let mut spans = vec![
        Span::styled(
            format!("{}   ", report.server_name),
            Style::default().fg(colors.muted),
        ),
        Span::styled(
            format!("{} transferred", format_bytes(transferred)),
            Style::default().fg(colors.muted),
        ),
    ];
    if let Some(ip_ver) = report.ip_version {
        spans.push(Span::styled(
            format!("   {}", ip_ver.label()),
            Style::default().fg(colors.muted),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(spans).centered()), area);
}

// ── Shared widget helpers ────────────────────────────────────────────────────

/// Large speed figure: label on top, value + unit in the middle, peak below.
fn render_speed_headline(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    stats: &TransferStats,
    value_color: Color,
    subtext: Color,
    speed_unit: SpeedUnit,
) {
    let (value, unit) = split_bps_unit(stats.average_bps, speed_unit);
    let (peak, peak_unit) = split_bps_unit(stats.peak_bps, speed_unit);
    let lines = vec![
        Line::from(Span::styled(label.to_owned(), Style::default().fg(subtext))).centered(),
        Line::from(vec![
            Span::styled(
                value,
                Style::default()
                    .fg(value_color)
                    .add_modifier(Modifier::BOLD),
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
