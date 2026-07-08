//! The latency metrics card.
//!
//! On the results screen (`active = false`) a compact histogram of ping
//! sample distribution is appended below the stats rows so users can
//! see at a glance whether latency was consistent or spiky.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Paragraph};
use ratatui::Frame;

use crate::app::state::PingView;
use crate::tui::theme::Theme;
use crate::utils::format::format_millis;

/// Block-character bar heights, index 0 (empty) through 8 (full).
const BAR: [&str; 9] = [" ", "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

/// Number of histogram buckets.
const BINS: usize = 8;

/// Minimum samples required to render a meaningful histogram.
const MIN_SAMPLES: usize = 3;

/// Renders the ping card: headline latency plus jitter, range and loss.
/// When `active` is false (results screen) and enough samples exist, a
/// one-line histogram of the latency distribution is appended.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, view: &PingView, active: bool) {
    if area.height == 0 {
        return;
    }
    let colors = &theme.colors;
    let border_color = if active {
        colors.latency
    } else {
        colors.border
    };
    let glyphs = crate::tui::glyphs::current();
    let block = Block::bordered()
        .border_set(glyphs.border)
        .border_style(Style::default().fg(border_color))
        .padding(Padding::horizontal(1))
        .title(Line::from(vec![
            Span::styled(
                format!(" {} ", glyphs.latency),
                Style::default().fg(colors.latency),
            ),
            Span::styled(
                "Ping ",
                Style::default()
                    .fg(colors.text)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let headline = view
        .stats
        .map(|stats| format_millis(stats.average_ms))
        .or_else(|| view.last_ms.map(format_millis))
        .unwrap_or_else(|| "—".to_owned());

    let (jitter, range, loss) = match view.stats {
        Some(stats) => (
            format_millis(stats.jitter_ms),
            format!(
                "{} – {}",
                format_millis(stats.min_ms),
                format_millis(stats.max_ms)
            ),
            format!("{:.0}%", stats.packet_loss_pct),
        ),
        None => ("—".to_owned(), "—".to_owned(), "—".to_owned()),
    };

    let label = Style::default().fg(colors.subtext);
    let value = Style::default().fg(colors.text);
    let mut lines = vec![
        Line::from(Span::styled(
            headline,
            Style::default()
                .fg(colors.latency)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("jitter  ", label),
            Span::styled(jitter, value),
        ]),
        Line::from(vec![
            Span::styled("range   ", label),
            Span::styled(range, value),
        ]),
        Line::from(vec![
            Span::styled("loss    ", label),
            Span::styled(loss, value),
        ]),
    ];

    if !active {
        if let Some(hist) = build_histogram_line(&view.history, colors.latency, colors.muted) {
            lines.push(hist);
        }
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

// ── Histogram ────────────────────────────────────────────────────────────────

/// Builds a single `Line` showing a block-char histogram of ping samples
/// followed by a `min – max` range label. Returns `None` when there are
/// not enough samples or all samples are identical.
fn build_histogram_line(
    history: &crate::engine::metrics::Sampler,
    bar_color: ratatui::style::Color,
    label_color: ratatui::style::Color,
) -> Option<Line<'static>> {
    if history.len() < MIN_SAMPLES {
        return None;
    }
    let lo = history.min()?;
    let hi = history.max()?;
    if hi - lo < 0.5 {
        return None;
    }

    let counts = bucket(history, lo, hi);
    let peak = counts.iter().copied().max().unwrap_or(1).max(1);
    let bars: String = counts
        .iter()
        .map(|&c| BAR[(c * 8 / peak).min(8) as usize])
        .collect();

    let range_label = format!("  {} – {}", format_millis(lo), format_millis(hi));

    Some(Line::from(vec![
        Span::styled(bars, Style::default().fg(bar_color)),
        Span::styled(range_label, Style::default().fg(label_color)),
    ]))
}

/// Distributes samples into `BINS` equally-spaced buckets between `lo` and
/// `hi`, returning the count in each bucket.
fn bucket(history: &crate::engine::metrics::Sampler, lo: f64, hi: f64) -> [u32; BINS] {
    let span = hi - lo;
    let mut counts = [0u32; BINS];
    for s in history.iter() {
        let idx = ((s - lo) / span * BINS as f64).floor() as usize;
        counts[idx.min(BINS - 1)] += 1;
    }
    counts
}
