//! Transfer metrics card: shared layout used by download and upload.

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Paragraph};
use ratatui::Frame;

use crate::app::state::TransferView;
use crate::tui::theme::Theme;
use crate::utils::format::{format_bps, format_bytes};

/// Identity and accent of a transfer card.
#[derive(Debug, Clone, Copy)]
pub struct CardStyle {
    /// Card title, e.g. `Download`.
    pub title: &'static str,
    /// Leading icon, e.g. `↓`.
    pub icon: &'static str,
    /// Accent color for the headline and active border.
    pub color: Color,
}

/// Renders one transfer card (used by the download and upload cards).
///
/// Shows the headline speed plus average, peak and transferred bytes.
/// `active` highlights the card while its phase is running.
pub fn render(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    style: CardStyle,
    view: &TransferView,
    active: bool,
) {
    if area.height == 0 {
        return;
    }
    let CardStyle { title, icon, color } = style;
    let colors = &theme.colors;
    let border_color = if active { color } else { colors.border };
    let block = Block::bordered()
        .border_set(crate::tui::glyphs::current().border)
        .border_style(Style::default().fg(border_color))
        .padding(Padding::horizontal(1))
        .title(Line::from(vec![
            Span::styled(format!(" {icon} "), Style::default().fg(color)),
            Span::styled(
                format!("{title} "),
                Style::default()
                    .fg(colors.text)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let headline_bps = view
        .stats
        .map_or(view.current_bps, |stats| stats.average_bps);
    let headline = if started(view) {
        format_bps(headline_bps)
    } else {
        "—".to_owned()
    };

    let lines = vec![
        Line::from(Span::styled(
            headline,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        metric_line(
            colors.subtext,
            colors.text,
            "avg ",
            value_or_dash(view, view.average_bps),
        ),
        metric_line(
            colors.subtext,
            colors.text,
            "peak",
            value_or_dash(view, view.peak_bps),
        ),
        metric_line(
            colors.subtext,
            colors.text,
            "data",
            if view.bytes > 0 {
                format_bytes(view.bytes)
            } else {
                "—".to_owned()
            },
        ),
    ];
    frame.render_widget(Paragraph::new(lines), inner);
}

/// Renders the download card.
pub fn render_download(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    view: &TransferView,
    active: bool,
) {
    render(
        frame,
        area,
        theme,
        CardStyle {
            title: "Download",
            icon: crate::tui::glyphs::current().down,
            color: theme.colors.download,
        },
        view,
        active,
    );
}

/// Renders the upload card.
pub fn render_upload(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    view: &TransferView,
    active: bool,
) {
    render(
        frame,
        area,
        theme,
        CardStyle {
            title: "Upload",
            icon: crate::tui::glyphs::current().up,
            color: theme.colors.upload,
        },
        view,
        active,
    );
}

/// Whether the phase has produced any data yet.
fn started(view: &TransferView) -> bool {
    view.bytes > 0 || view.stats.is_some()
}

/// Formats a speed metric, or an em dash before the phase starts.
fn value_or_dash(view: &TransferView, bps: f64) -> String {
    if started(view) {
        format_bps(bps)
    } else {
        "—".to_owned()
    }
}

/// One `label value` metric row.
fn metric_line(
    label_color: Color,
    value_color: Color,
    label: &str,
    value: String,
) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}  "), Style::default().fg(label_color)),
        Span::styled(value, Style::default().fg(value_color)),
    ])
}
