//! A hypercar-style tachometer for network speed.
//!
//! Rendered with ratatui's [`Canvas`] in Braille marker mode (a 2×4 dot
//! grid per terminal cell). The face borrows from performance-car
//! instrument clusters: a thick value band that heats from the phase
//! color toward red as it climbs, a hatched redline, numerals set inside
//! the face, a tapered needle with a counterweight tail, bright tip and
//! fading afterglow trail, a telemetry-style ghost notch holding the
//! session peak, and a small latency sub-dial inset in the face. The
//! needle sweeps smoothly because callers feed it the renderer's eased
//! speeds; on phase start the renderer drives a full ignition sweep
//! through [`DialData::override_ratio`].

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::canvas::{Canvas, Context, Line as CanvasLine, Points};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::theme::{blend, Colors, Theme};
use crate::utils::format::{split_bps_unit, SpeedUnit};

use super::digits;

/// Angle of the zero position, in degrees (7 o'clock).
const START_DEG: f64 = 210.0;
/// Total sweep of the dial, in degrees (ends at 5 o'clock).
const SWEEP_DEG: f64 = 240.0;
/// Fraction of the sweep drawn as the redline zone.
const REDLINE_FROM: f64 = 0.85;
/// Sweep fraction where the value band starts heating toward red.
const HEAT_FROM: f64 = 0.60;
/// Number of major (labelled) tick intervals.
const MAJOR_TICKS: usize = 5;
/// Base dot samples along a full arc; scaled up on larger canvases.
const ARC_SAMPLES: usize = 240;
/// Gradient resolution of the value band.
const BAND_SEGMENTS: usize = 30;
/// Radii of the value band rings (thickness of the lit arc).
const BAND_RADII: [f64; 5] = [0.90, 0.93, 0.96, 0.99, 1.02];
/// Latency that pins the sub-dial, in milliseconds.
const PING_SCALE_MS: f64 = 300.0;
/// Minimum dial height that earns the block-digit readout.
const BIG_READOUT_MIN_HEIGHT: u16 = 19;
/// One megabit, in bits.
const MBPS: f64 = 1_000_000.0;

/// Everything one dial displays.
#[derive(Debug, Clone, Copy)]
pub struct DialData<'a> {
    /// Phase label shown inside the face, e.g. `Download`.
    pub label: &'a str,
    /// Current (eased) speed, in bits per second.
    pub bps: f64,
    /// Phase accent color.
    pub color: Color,
    /// Session peak, in bits per second; drives the scale and the ghost
    /// notch.
    pub peak_bps: f64,
    /// Recent displayed speeds (oldest → newest) for the afterglow trail.
    pub trail: &'a [f64],
    /// Latency shown on the inset sub-dial, when known.
    pub ping_ms: Option<f64>,
    /// Overrides the needle position (`0.0..=1.0`) during the ignition
    /// sweep.
    pub override_ratio: Option<f64>,
    /// Renders the dial in a muted palette (inactive twin in a cluster).
    pub dimmed: bool,
    /// Whether to display speeds in Mbps or MB/s.
    pub speed_unit: SpeedUnit,
}

/// Picks a round dial maximum, in bits per second, for an observed peak.
///
/// Steps follow the 1 / 2.5 / 5 decade pattern with a floor of 100 Mbps.
/// Peak speed is monotonic within a phase, so the returned scale only
/// ever grows — the needle never pins at the end of the dial.
#[must_use]
pub fn nice_max_bps(peak_bps: f64) -> f64 {
    let peak_mbps = if peak_bps.is_finite() && peak_bps > 0.0 {
        peak_bps / MBPS
    } else {
        0.0
    };
    let needed = peak_mbps * 1.02;
    let mut step = 100.0;
    let mut index = 0_usize;
    const PATTERN: [f64; 3] = [1.0, 2.5, 5.0];
    let mut decade = 100.0;
    while step < needed {
        index += 1;
        decade *= if index.is_multiple_of(PATTERN.len()) {
            10.0
        } else {
            1.0
        };
        step = PATTERN[index % PATTERN.len()] * decade;
    }
    step * MBPS
}

/// Maps a value ratio (`0.0..=1.0`, clamped) to its needle angle in
/// degrees: 210° at zero, sweeping clockwise 240° down to −30° at max.
#[must_use]
pub fn value_angle_deg(ratio: f64) -> f64 {
    START_DEG - SWEEP_DEG * ratio.clamp(0.0, 1.0)
}

/// The color of the value band at sweep position `t`: the phase color,
/// heating smoothly toward the danger color past [`HEAT_FROM`].
#[must_use]
pub fn band_color(t: f64, phase: Color, danger: Color) -> Color {
    let heat = ((t - HEAT_FROM) / (1.0 - HEAT_FROM)).clamp(0.0, 1.0);
    blend(phase, danger, heat * heat)
}

/// The resolved palette for one dial, dimmed as a unit for inactive
/// twins.
struct DialPalette {
    phase: Color,
    danger: Color,
    warning: Color,
    frame: Color,
    marks: Color,
    caption: Color,
    hub: Color,
}

impl DialPalette {
    fn new(colors: &Colors, phase: Color, dimmed: bool) -> Self {
        let dim = |color: Color| {
            if dimmed {
                blend(color, colors.background, 0.65)
            } else {
                color
            }
        };
        Self {
            phase: dim(phase),
            danger: dim(colors.danger),
            warning: dim(colors.warning),
            frame: dim(colors.border),
            marks: dim(colors.subtext),
            caption: dim(colors.muted),
            hub: dim(colors.text),
        }
    }
}

/// Renders the tachometer and its numeric readout.
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, data: &DialData) {
    if area.height < 6 || area.width < 24 {
        return;
    }
    let colors = &theme.colors;
    let palette = DialPalette::new(colors, data.color, data.dimmed);
    let readout_height = if area.height >= BIG_READOUT_MIN_HEIGHT {
        digits::FONT_HEIGHT as u16
    } else {
        1
    };
    let [canvas_area, readout_area] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(readout_height)]).areas(area);

    let max_bps = nice_max_bps(data.peak_bps.max(data.bps));
    let live_ratio = (data.bps / max_bps).clamp(0.0, 1.0);
    let ratio = data
        .override_ratio
        .map_or(live_ratio, |r| r.clamp(0.0, 1.0));
    let peak_ratio = (data.peak_bps / max_bps).clamp(0.0, 1.0);

    // Larger canvases get denser arcs, so 4K terminals stay smooth.
    let samples = ARC_SAMPLES * (usize::from(canvas_area.width) / 40).clamp(1, 4);

    // Keep the dial circular: braille cells are 2 dots wide and 4 tall,
    // so equal units-per-dot means x-extent scales with width/(2*height).
    let y_min = -0.72;
    let y_max = 1.26;
    let x_half = ((y_max - y_min) * f64::from(canvas_area.width)
        / (2.0 * f64::from(canvas_area.height.max(1))))
    .max(1.35)
        / 2.0;
    let char_width = x_half * 2.0 / f64::from(canvas_area.width.max(1));

    // Geometry is precomputed so the paint closure only borrows.
    let bezel: Vec<(f64, f64)> = arc_points(0.0, 1.0, 1.10, samples);
    let track: Vec<(f64, f64)> = arc_points(ratio, 1.0, 0.96, samples);
    let band = band_segments(ratio, palette.phase, palette.danger, samples);
    let hatching = redline_hatching();
    let hub_ring: Vec<(f64, f64)> = arc_points(0.0, 1.0, 0.07, samples);
    let hub_core: Vec<(f64, f64)> = arc_points(0.0, 1.0, 0.025, samples);
    let trail = trail_points(data.trail, max_bps, colors.background, palette.phase);
    let max_mbps = max_bps / MBPS;

    let canvas = Canvas::default()
        .marker(Marker::Braille)
        .x_bounds([-x_half, x_half])
        .y_bounds([y_min, y_max])
        .paint(|ctx| {
            // Bezel and unlit track give the face its depth.
            ctx.draw(&Points {
                coords: &bezel,
                color: palette.frame,
            });
            ctx.draw(&Points {
                coords: &track,
                color: palette.frame,
            });

            // The lit band, heating toward the redline.
            for (coords, segment_color) in &band {
                ctx.draw(&Points {
                    coords,
                    color: *segment_color,
                });
            }

            // Hatched redline across the full band thickness.
            for (x1, y1, x2, y2) in &hatching {
                ctx.draw(&CanvasLine {
                    x1: *x1,
                    y1: *y1,
                    x2: *x2,
                    y2: *y2,
                    color: palette.danger,
                });
            }

            draw_ticks(ctx, palette.marks, palette.frame);
            draw_labels(ctx, max_mbps, char_width, palette.marks);

            // Ghost notch at the session peak, telemetry-style.
            if peak_ratio > 0.01 {
                let angle = value_angle_deg(peak_ratio);
                let (x1, y1) = polar(angle, 0.90);
                let (x2, y2) = polar(angle, 1.07);
                ctx.draw(&CanvasLine {
                    x1,
                    y1,
                    x2,
                    y2,
                    color: palette.warning,
                });
            }

            // Face captions, like a cluster's "RPM ×1000".
            ctx.print(
                -char_width * data.label.len() as f64 / 2.0,
                -0.28,
                Line::from(Span::styled(
                    data.label.to_uppercase(),
                    Style::default()
                        .fg(palette.marks)
                        .add_modifier(Modifier::BOLD),
                )),
            );
            ctx.print(
                -char_width * 2.0,
                -0.46,
                Line::from(Span::styled("Mbps", Style::default().fg(palette.caption))),
            );

            if !data.dimmed {
                draw_ping_inset(ctx, data.ping_ms, char_width, &palette, colors);
            }

            // Afterglow trail behind the needle.
            for (coords, glow_color) in &trail {
                ctx.draw(&Points {
                    coords,
                    color: *glow_color,
                });
            }

            draw_needle(ctx, ratio, palette.danger, palette.warning);
            ctx.draw(&Points {
                coords: &hub_ring,
                color: palette.hub,
            });
            ctx.draw(&Points {
                coords: &hub_core,
                color: palette.danger,
            });
        });
    frame.render_widget(canvas, canvas_area);

    render_readout(frame, readout_area, data, &palette, colors);
}

/// The numeric readout under the face: block digits on tall dials, a
/// single styled line otherwise.
fn render_readout(
    frame: &mut Frame,
    area: Rect,
    data: &DialData,
    palette: &DialPalette,
    colors: &Colors,
) {
    let (value, unit) = split_bps_unit(data.bps, data.speed_unit);
    if area.height >= digits::FONT_HEIGHT as u16 {
        digits::render_value(frame, area, &value, unit, palette.phase, colors.muted);
        return;
    }
    let readout = Line::from(vec![
        Span::styled(
            value,
            Style::default()
                .fg(palette.phase)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {unit}"), Style::default().fg(palette.marks)),
    ])
    .centered();
    frame.render_widget(Paragraph::new(readout), area);
}

/// Converts a dial angle (degrees) and radius to canvas coordinates.
fn polar(angle_deg: f64, radius: f64) -> (f64, f64) {
    let radians = angle_deg.to_radians();
    (radius * radians.cos(), radius * radians.sin())
}

/// Samples dots along the arc between two sweep ratios.
fn arc_points(from: f64, to: f64, radius: f64, samples: usize) -> Vec<(f64, f64)> {
    let (from, to) = (from.clamp(0.0, 1.0), to.clamp(0.0, 1.0));
    if to <= from {
        return Vec::new();
    }
    let count = ((to - from) * samples as f64).ceil().max(2.0) as usize;
    (0..=count)
        .map(|step| {
            let t = from + (to - from) * step as f64 / count as f64;
            polar(value_angle_deg(t), radius)
        })
        .collect()
}

/// Builds the lit value band as gradient segments across all band radii.
fn band_segments(
    ratio: f64,
    phase: Color,
    danger: Color,
    samples: usize,
) -> Vec<(Vec<(f64, f64)>, Color)> {
    let lit = (ratio * BAND_SEGMENTS as f64).ceil() as usize;
    (0..lit)
        .map(|segment| {
            let from = segment as f64 / BAND_SEGMENTS as f64;
            let to = ((segment + 1) as f64 / BAND_SEGMENTS as f64).min(ratio);
            let coords = BAND_RADII
                .iter()
                .flat_map(|radius| arc_points(from, to, *radius, samples))
                .collect();
            (coords, band_color(from, phase, danger))
        })
        .collect()
}

/// Diagonal hatch strokes across the redline zone of the band.
fn redline_hatching() -> Vec<(f64, f64, f64, f64)> {
    const STROKES: usize = 7;
    (0..STROKES)
        .map(|stroke| {
            let t = REDLINE_FROM + (1.0 - REDLINE_FROM) * stroke as f64 / (STROKES - 1) as f64;
            let lean = 0.018;
            let (x1, y1) = polar(value_angle_deg((t - lean).max(0.0)), BAND_RADII[0]);
            let (x2, y2) = polar(value_angle_deg((t + lean).min(1.0)), 1.06);
            (x1, y1, x2, y2)
        })
        .collect()
}

/// Fading dot clusters at the needle's recent positions.
///
/// Older samples blend further toward the background, producing a motion
/// blur behind the moving needle.
fn trail_points(
    trail: &[f64],
    max_bps: f64,
    background: Color,
    phase: Color,
) -> Vec<(Vec<(f64, f64)>, Color)> {
    let count = trail.len();
    trail
        .iter()
        .enumerate()
        .map(|(index, bps)| {
            let ratio = (bps / max_bps).clamp(0.0, 1.0);
            let angle = value_angle_deg(ratio);
            let coords = vec![polar(angle, 0.80), polar(angle, 0.83)];
            // Newest sample (last) glows strongest.
            let age = 1.0 - (index as f64 + 1.0) / count as f64;
            (coords, blend(phase, background, 0.35 + age * 0.55))
        })
        .collect()
}

/// Major and minor ticks, set inside the face pointing at the band.
fn draw_ticks(ctx: &mut Context, major_color: Color, minor_color: Color) {
    for major in 0..=MAJOR_TICKS {
        let t = major as f64 / MAJOR_TICKS as f64;
        let angle = value_angle_deg(t);
        let (x1, y1) = polar(angle, 0.78);
        let (x2, y2) = polar(angle, 0.87);
        ctx.draw(&CanvasLine {
            x1,
            y1,
            x2,
            y2,
            color: major_color,
        });
        if major < MAJOR_TICKS {
            for minor in 1..5 {
                let mt = t + f64::from(minor) / (MAJOR_TICKS as f64 * 5.0);
                let angle = value_angle_deg(mt);
                let (x1, y1) = polar(angle, 0.83);
                let (x2, y2) = polar(angle, 0.87);
                ctx.draw(&CanvasLine {
                    x1,
                    y1,
                    x2,
                    y2,
                    color: minor_color,
                });
            }
        }
    }
}

/// Numerals inside the face, cockpit-style.
fn draw_labels(ctx: &mut Context, max_mbps: f64, char_width: f64, color: Color) {
    for major in 0..=MAJOR_TICKS {
        let t = major as f64 / MAJOR_TICKS as f64;
        let text = tick_label(max_mbps * t);
        let (lx, ly) = polar(value_angle_deg(t), 0.62);
        ctx.print(
            lx - char_width * text.len() as f64 / 2.0,
            ly,
            Line::from(Span::styled(text, Style::default().fg(color))),
        );
    }
}

/// A small fuel-gauge style latency dial inset in the lower-right of the
/// face: a half-circle scale pinned at [`PING_SCALE_MS`].
fn draw_ping_inset(
    ctx: &mut Context,
    ping_ms: Option<f64>,
    char_width: f64,
    palette: &DialPalette,
    colors: &Colors,
) {
    let Some(ms) = ping_ms else {
        return;
    };
    let center = (0.56, -0.34);
    let radius = 0.20;
    // Half-circle from 180° (0 ms) to 0° (PING_SCALE_MS).
    let arc: Vec<(f64, f64)> = (0..=40)
        .map(|step| {
            let angle = 180.0 - 180.0 * f64::from(step) / 40.0;
            offset_polar(center, angle, radius)
        })
        .collect();
    ctx.draw(&Points {
        coords: &arc,
        color: palette.frame,
    });

    let ratio = (ms / PING_SCALE_MS).clamp(0.0, 1.0);
    let angle = 180.0 - 180.0 * ratio;
    let (nx, ny) = offset_polar(center, angle, radius * 0.85);
    ctx.draw(&CanvasLine {
        x1: center.0,
        y1: center.1,
        x2: nx,
        y2: ny,
        color: colors.latency,
    });

    let text = format!("{ms:.0}ms");
    ctx.print(
        center.0 - char_width * text.len() as f64 / 2.0,
        center.1 - 0.16,
        Line::from(Span::styled(text, Style::default().fg(palette.caption))),
    );
}

/// [`polar`] around an arbitrary center.
fn offset_polar(center: (f64, f64), angle_deg: f64, radius: f64) -> (f64, f64) {
    let (x, y) = polar(angle_deg, radius);
    (center.0 + x, center.1 + y)
}

/// The tapered needle: weighted shaft, counterweight tail, bright tip.
fn draw_needle(ctx: &mut Context, ratio: f64, shaft_color: Color, tip_color: Color) {
    let angle = value_angle_deg(ratio);
    // Wider near the hub, converging at the tip.
    for offset in [-1.6_f64, -0.8, 0.0, 0.8, 1.6] {
        let reach = 0.86 - offset.abs() * 0.04;
        let (nx, ny) = polar(angle + offset, reach);
        let (bx, by) = polar(angle, 0.05);
        ctx.draw(&CanvasLine {
            x1: bx,
            y1: by,
            x2: nx,
            y2: ny,
            color: shaft_color,
        });
    }
    // Counterweight tail opposite the shaft.
    let (tx, ty) = polar(angle + 180.0, 0.16);
    ctx.draw(&CanvasLine {
        x1: 0.0,
        y1: 0.0,
        x2: tx,
        y2: ty,
        color: shaft_color,
    });
    // Bright tip.
    let tip: Vec<(f64, f64)> = (0..3)
        .map(|step| polar(angle, 0.84 + f64::from(step) * 0.015))
        .collect();
    ctx.draw(&Points {
        coords: &tip,
        color: tip_color,
    });
}

/// Formats a tick value in Mbps, switching to a compact gigabit form for
/// large scales (`1.5G`).
fn tick_label(mbps: f64) -> String {
    if mbps >= 1000.0 {
        let gbps = mbps / 1000.0;
        if (gbps - gbps.round()).abs() < 1e-9 {
            format!("{gbps:.0}G")
        } else {
            format!("{gbps:.1}G")
        }
    } else {
        format!("{mbps:.0}")
    }
}
