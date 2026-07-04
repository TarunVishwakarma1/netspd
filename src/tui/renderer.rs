//! Top-level frame composition: background, header, screen body, footer.

use std::collections::VecDeque;
use std::time::Instant;

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::Frame;

use crate::app::state::{AppState, Screen};
use crate::engine::models::TestPhase;

use super::animation::{AnimatedValue, SpringValue};
use super::screens;
use super::theme::Theme;
use super::widgets::footer::{self, Hint};
use super::widgets::header;

/// Spring stiffness for the needle: how hard it pulls toward the value.
const NEEDLE_STIFFNESS: f64 = 60.0;
/// Spring damping: below critical, so the needle overshoots slightly.
const NEEDLE_DAMPING: f64 = 9.0;
/// Base approach rate for progress ratios.
const RATIO_RATE: f64 = 10.0;
/// Needle positions kept for the afterglow trail.
const TRAIL_LENGTH: usize = 8;

/// Duration of the ignition sweep played when a transfer phase starts.
///
/// Runs in real time (never scaled by the animation speed) and must match
/// the engine's default transfer lead-in, so the sweep finishes exactly
/// as the first bytes move.
pub const SWEEP_SECONDS: f64 = 1.2;

/// Smoothed display values derived from raw state each frame.
#[derive(Debug, Clone)]
pub struct Motion {
    /// Spring-animated download speed, in bits per second.
    pub download_bps: f64,
    /// Spring-animated upload speed, in bits per second.
    pub upload_bps: f64,
    /// Animated download progress ratio.
    pub download_ratio: f64,
    /// Animated upload progress ratio.
    pub upload_ratio: f64,
    /// Recent displayed download speeds (oldest → newest).
    pub download_trail: Vec<f64>,
    /// Recent displayed upload speeds (oldest → newest).
    pub upload_trail: Vec<f64>,
    /// Needle position of the ignition sweep, while one is playing.
    pub sweep_ratio: Option<f64>,
}

/// Animation state for one dial's needle.
struct NeedleAnim {
    spring: SpringValue,
    trail: VecDeque<f64>,
}

impl NeedleAnim {
    fn new() -> Self {
        Self {
            spring: SpringValue::new(NEEDLE_STIFFNESS, NEEDLE_DAMPING),
            trail: VecDeque::with_capacity(TRAIL_LENGTH),
        }
    }

    fn reset(&mut self) {
        self.spring.snap(0.0);
        self.trail.clear();
    }

    fn advance(&mut self, target: f64, dt: f64) -> f64 {
        self.spring.set_target(target);
        let value = self.spring.tick(dt).max(0.0);
        if self.trail.len() == TRAIL_LENGTH {
            self.trail.pop_front();
        }
        self.trail.push_back(value);
        value
    }
}

/// The stateful renderer: owns the loaded themes and the animation state
/// that eases displayed numbers toward their true values.
pub struct Renderer {
    themes: Vec<Theme>,
    animation_speed: f64,
    download: NeedleAnim,
    upload: NeedleAnim,
    download_ratio: AnimatedValue,
    upload_ratio: AnimatedValue,
    last_phase: Option<TestPhase>,
    sweep_started: Option<Instant>,
}

impl Renderer {
    /// Creates a renderer over the loaded themes.
    ///
    /// `animation_speed` scales how quickly displayed values chase their
    /// targets.
    #[must_use]
    pub fn new(themes: Vec<Theme>, animation_speed: f64) -> Self {
        Self {
            themes,
            animation_speed,
            download: NeedleAnim::new(),
            upload: NeedleAnim::new(),
            download_ratio: AnimatedValue::new(RATIO_RATE * animation_speed),
            upload_ratio: AnimatedValue::new(RATIO_RATE * animation_speed),
            last_phase: None,
            sweep_started: None,
        }
    }

    /// Names of all loaded themes, in selection order.
    #[must_use]
    pub fn theme_names(&self) -> Vec<String> {
        self.themes.iter().map(|theme| theme.name.clone()).collect()
    }

    /// Renders one complete frame from the current state.
    pub fn render(&mut self, frame: &mut Frame, state: &AppState, dt: f64) {
        let motion = self.advance(state, dt);
        let theme = self.theme(state.theme_index);
        let area = frame.area();

        frame.render_widget(
            Block::default().style(Style::default().bg(theme.colors.background)),
            area,
        );

        let [header_area, body_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .areas(area);

        header::render(
            frame,
            header_area,
            &theme,
            &format!("{} · {}", state.provider_name, state.server_name()),
            state.client_info.as_deref(),
        );
        self.render_body(frame, body_area, &theme, state, &motion);
        footer::render(frame, footer_area, &theme, hints_for(state.screen));
    }

    /// Advances animations toward the state's raw values.
    fn advance(&mut self, state: &AppState, dt: f64) -> Motion {
        let dt = dt * self.animation_speed;

        // Phase transitions reset the affected needle and trigger the
        // ignition sweep on transfer phases.
        if state.phase != self.last_phase {
            match state.phase {
                Some(TestPhase::Download) => {
                    self.download.reset();
                    self.sweep_started = Some(Instant::now());
                }
                Some(TestPhase::Upload) => {
                    self.upload.reset();
                    self.sweep_started = Some(Instant::now());
                }
                _ => self.sweep_started = None,
            }
            self.last_phase = state.phase;
        }

        let sweep_ratio = self.sweep_started.and_then(|started| {
            // Real time, not animation time: the engine's lead-in pause is
            // wall-clock, and the two must land together.
            let t = started.elapsed().as_secs_f64() / SWEEP_SECONDS;
            if t >= 1.0 {
                self.sweep_started = None;
                None
            } else {
                Some(sweep_position(t))
            }
        });

        let download_bps = self.download.advance(state.download.current_bps, dt);
        let upload_bps = self.upload.advance(state.upload.current_bps, dt);
        self.download_ratio.set_target(state.download.ratio);
        self.upload_ratio.set_target(state.upload.ratio);

        Motion {
            download_bps,
            upload_bps,
            download_ratio: self.download_ratio.tick(dt).clamp(0.0, 1.0),
            upload_ratio: self.upload_ratio.tick(dt).clamp(0.0, 1.0),
            download_trail: self.download.trail.iter().copied().collect(),
            upload_trail: self.upload.trail.iter().copied().collect(),
            sweep_ratio,
        }
    }

    /// The active theme, clamped to a valid index.
    fn theme(&self, index: usize) -> Theme {
        let index = index.min(self.themes.len().saturating_sub(1));
        self.themes
            .get(index)
            .cloned()
            .unwrap_or_else(fallback_theme)
    }

    /// Renders the screen body; overlays draw on top of their parent.
    fn render_body(
        &self,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
        state: &AppState,
        motion: &Motion,
    ) {
        let screen = state.screen;
        if screen.is_overlay() {
            self.render_base(frame, area, theme, state, motion, state.return_to);
        }
        match screen {
            Screen::Help => screens::help::render(frame, area, theme),
            Screen::Settings => screens::settings::render(frame, area, theme, state),
            Screen::ServerSelect => screens::server_select::render(frame, area, theme, state),
            Screen::ThemeSelect => {
                screens::theme_select::render(frame, area, theme, state, &self.themes);
            }
            Screen::Trends => screens::trends::render(frame, area, theme, state),
            base => self.render_base(frame, area, theme, state, motion, base),
        }
    }

    /// Renders one of the non-overlay screens.
    fn render_base(
        &self,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
        state: &AppState,
        motion: &Motion,
        screen: Screen,
    ) {
        match screen {
            Screen::Splash => {
                let status = if state.servers_loading {
                    "locating servers…"
                } else {
                    "starting test…"
                };
                screens::splash::render(frame, area, theme, state.tick, status);
            }
            Screen::Testing => screens::testing::render(frame, area, theme, state, motion),
            Screen::Results => screens::results::render(frame, area, theme, state),
            Screen::Error => {
                let message = state.error.as_deref().unwrap_or("unknown error");
                screens::error::render(frame, area, theme, message);
            }
            // Overlays never reach here; fall back to the splash rather
            // than drawing nothing.
            _ => screens::splash::render(frame, area, theme, state.tick, ""),
        }
    }
}

/// The needle position during the ignition sweep at normalized time
/// `t ∈ 0..1`: an eased rise to full scale, then an eased fall back.
fn sweep_position(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    let leg = if t < 0.5 { t / 0.5 } else { (1.0 - t) / 0.5 };
    // Smoothstep for a mechanical, non-linear throw.
    leg * leg * (3.0 - 2.0 * leg)
}

/// The footer hints for each screen.
fn hints_for(screen: Screen) -> &'static [Hint] {
    match screen {
        Screen::Splash => &[("q", "quit")],
        Screen::Testing => &[
            ("q", "quit"),
            ("r", "restart"),
            ("s", "servers"),
            ("t", "theme"),
            ("?", "help"),
        ],
        Screen::Results => &[
            ("r", "run again"),
            ("g", "trends"),
            ("s", "servers"),
            ("t", "theme"),
            ("q", "quit"),
        ],
        Screen::Help | Screen::Settings | Screen::Trends => &[("Esc", "back"), ("q", "quit")],
        Screen::ServerSelect | Screen::ThemeSelect => {
            &[("↑↓", "navigate"), ("Enter", "select"), ("Esc", "back")]
        }
        Screen::Error => &[("r", "retry"), ("s", "servers"), ("q", "quit")],
    }
}

/// A minimal, always-available theme used only if no theme loaded.
fn fallback_theme() -> Theme {
    use ratatui::style::Color;
    let gray = Color::Rgb(160, 160, 170);
    Theme {
        name: "Fallback".to_owned(),
        colors: super::theme::Colors {
            background: Color::Rgb(20, 20, 26),
            surface: Color::Rgb(30, 30, 38),
            overlay: Color::Rgb(40, 40, 50),
            text: Color::Rgb(220, 220, 228),
            subtext: gray,
            muted: Color::Rgb(100, 100, 112),
            border: Color::Rgb(60, 60, 72),
            accent: Color::Rgb(122, 162, 247),
            accent_alt: Color::Rgb(187, 154, 247),
            success: Color::Rgb(158, 206, 106),
            warning: Color::Rgb(224, 175, 104),
            danger: Color::Rgb(247, 118, 142),
            download: Color::Rgb(125, 207, 255),
            upload: Color::Rgb(187, 154, 247),
            latency: Color::Rgb(158, 206, 106),
        },
    }
}
