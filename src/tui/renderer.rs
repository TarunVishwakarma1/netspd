//! Top-level frame composition: background, header, screen body, footer.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::Frame;

use crate::app::state::{AppState, Screen};

use super::animation::AnimatedValue;
use super::screens;
use super::theme::Theme;
use super::widgets::footer::{self, Hint};
use super::widgets::header;

/// Base approach rate for speed counters.
const SPEED_RATE: f64 = 6.0;
/// Base approach rate for progress ratios (snappier than counters).
const RATIO_RATE: f64 = 10.0;

/// Smoothed display values derived from raw state each frame.
#[derive(Debug, Clone, Copy)]
pub struct Motion {
    /// Animated download speed, in bits per second.
    pub download_bps: f64,
    /// Animated upload speed, in bits per second.
    pub upload_bps: f64,
    /// Animated download progress ratio.
    pub download_ratio: f64,
    /// Animated upload progress ratio.
    pub upload_ratio: f64,
}

/// The stateful renderer: owns the loaded themes and the animation state
/// that eases displayed numbers toward their true values.
pub struct Renderer {
    themes: Vec<Theme>,
    download_bps: AnimatedValue,
    upload_bps: AnimatedValue,
    download_ratio: AnimatedValue,
    upload_ratio: AnimatedValue,
}

impl Renderer {
    /// Creates a renderer over the loaded themes.
    ///
    /// `animation_speed` scales how quickly displayed values chase their
    /// targets.
    #[must_use]
    pub fn new(themes: Vec<Theme>, animation_speed: f64) -> Self {
        let speed_rate = SPEED_RATE * animation_speed;
        let ratio_rate = RATIO_RATE * animation_speed;
        Self {
            themes,
            download_bps: AnimatedValue::new(speed_rate),
            upload_bps: AnimatedValue::new(speed_rate),
            download_ratio: AnimatedValue::new(ratio_rate),
            upload_ratio: AnimatedValue::new(ratio_rate),
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
        );
        self.render_body(frame, body_area, &theme, state, &motion);
        footer::render(frame, footer_area, &theme, hints_for(state.screen));
    }

    /// Advances animations toward the state's raw values.
    fn advance(&mut self, state: &AppState, dt: f64) -> Motion {
        self.download_bps.set_target(state.download.current_bps);
        self.upload_bps.set_target(state.upload.current_bps);
        self.download_ratio.set_target(state.download.ratio);
        self.upload_ratio.set_target(state.upload.ratio);
        Motion {
            download_bps: self.download_bps.tick(dt),
            upload_bps: self.upload_bps.tick(dt),
            download_ratio: self.download_ratio.tick(dt).clamp(0.0, 1.0),
            upload_ratio: self.upload_ratio.tick(dt).clamp(0.0, 1.0),
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
            ("s", "servers"),
            ("t", "theme"),
            ("c", "config"),
            ("q", "quit"),
        ],
        Screen::Help | Screen::Settings => &[("Esc", "back"), ("q", "quit")],
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
