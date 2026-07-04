//! Application state and the reducers that mutate it.
//!
//! State is plain data with no networking or rendering concerns, which
//! makes every transition unit-testable without a terminal or a socket.

use std::time::{Duration, Instant};

use crate::config::Settings;
use crate::engine::metrics::Sampler;
use crate::engine::models::{LatencyStats, Server, TestPhase, TestReport, TransferStats};
use crate::engine::EngineEvent;

/// Number of speed samples kept for sparklines.
const HISTORY_CAPACITY: usize = 120;

/// The screen currently in the foreground.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Startup splash while servers are discovered.
    Splash,
    /// A test is running.
    Testing,
    /// Final results of a completed test.
    Results,
    /// Keyboard shortcut reference.
    Help,
    /// Read-only view of the active configuration.
    Settings,
    /// Server selection list.
    ServerSelect,
    /// Theme selection list.
    ThemeSelect,
    /// Past results and trends.
    Trends,
    /// A fatal test error.
    Error,
}

impl Screen {
    /// Whether this screen is an overlay that returns to a parent screen.
    #[must_use]
    pub fn is_overlay(self) -> bool {
        matches!(
            self,
            Self::Help | Self::Settings | Self::ServerSelect | Self::ThemeSelect | Self::Trends
        )
    }

    /// Whether this screen animates continuously and needs steady redraws.
    #[must_use]
    pub fn is_animated(self) -> bool {
        matches!(self, Self::Splash | Self::Testing)
    }
}

/// Live view of the ping phase.
#[derive(Debug, Clone)]
pub struct PingView {
    /// Most recent sample, in milliseconds.
    pub last_ms: Option<f64>,
    /// Number of samples completed so far.
    pub samples_done: u32,
    /// Final statistics, once the phase completes.
    pub stats: Option<LatencyStats>,
    /// Recent samples for the sparkline.
    pub history: Sampler,
}

impl PingView {
    fn new() -> Self {
        Self {
            last_ms: None,
            samples_done: 0,
            stats: None,
            history: Sampler::new(HISTORY_CAPACITY),
        }
    }

    fn reset(&mut self) {
        self.last_ms = None;
        self.samples_done = 0;
        self.stats = None;
        self.history.clear();
    }
}

/// Live view of a download or upload phase.
#[derive(Debug, Clone)]
pub struct TransferView {
    /// Bytes transferred so far.
    pub bytes: u64,
    /// Smoothed instantaneous speed, in bits per second.
    pub current_bps: f64,
    /// Average speed, in bits per second.
    pub average_bps: f64,
    /// Peak smoothed speed, in bits per second.
    pub peak_bps: f64,
    /// Estimated time remaining in this phase.
    pub eta: Duration,
    /// Phase completion in `0.0..=1.0`.
    pub ratio: f64,
    /// Recent speed samples for the sparkline.
    pub history: Sampler,
    /// Final statistics, once the phase completes.
    pub stats: Option<TransferStats>,
}

impl TransferView {
    fn new() -> Self {
        Self {
            bytes: 0,
            current_bps: 0.0,
            average_bps: 0.0,
            peak_bps: 0.0,
            eta: Duration::ZERO,
            ratio: 0.0,
            history: Sampler::new(HISTORY_CAPACITY),
            stats: None,
        }
    }

    fn reset(&mut self) {
        self.bytes = 0;
        self.current_bps = 0.0;
        self.average_bps = 0.0;
        self.peak_bps = 0.0;
        self.eta = Duration::ZERO;
        self.ratio = 0.0;
        self.history.clear();
        self.stats = None;
    }
}

/// The complete state of the application.
#[derive(Debug)]
pub struct AppState {
    /// The screen in the foreground.
    pub screen: Screen,
    /// Where overlay screens return on [`Screen::is_overlay`] exit.
    pub return_to: Screen,
    /// The phase currently executing, if a test is running.
    pub phase: Option<TestPhase>,
    /// Ping phase view.
    pub ping: PingView,
    /// Download phase view.
    pub download: TransferView,
    /// Upload phase view.
    pub upload: TransferView,
    /// The last completed report.
    pub report: Option<TestReport>,
    /// Discovered servers.
    pub servers: Vec<Server>,
    /// Index of the active server in [`AppState::servers`].
    pub server_index: usize,
    /// Highlighted row on the server selection screen.
    pub server_cursor: usize,
    /// Names of all loaded themes.
    pub theme_names: Vec<String>,
    /// Index of the active theme.
    pub theme_index: usize,
    /// Highlighted row on the theme selection screen.
    pub theme_cursor: usize,
    /// Fatal error message, when on [`Screen::Error`].
    pub error: Option<String>,
    /// Whether server discovery is still in flight.
    pub servers_loading: bool,
    /// Whether a test task is currently running.
    pub testing: bool,
    /// Monotonic UI tick counter, drives spinners.
    pub tick: u64,
    /// When the current test started.
    pub started_at: Option<Instant>,
    /// Elapsed time of the current or last test.
    pub elapsed: Duration,
    /// When the splash screen appeared.
    pub splash_since: Instant,
    /// The loaded configuration (displayed on the settings screen).
    pub settings: Settings,
    /// Display name of the active provider.
    pub provider_name: &'static str,
    /// The client's public IP and ISP, once discovered.
    pub client_info: Option<String>,
    /// Server query from `--server`; applied when discovery completes.
    pub preferred_server: Option<String>,
    /// Auto-restart the test this long after completion, when set.
    pub repeat_every: Option<Duration>,
    /// When the last test completed; drives the repeat countdown.
    pub finished_at: Option<Instant>,
    /// Past results shown on the trends screen.
    pub trends: Vec<crate::app::history::HistoryRecord>,
    /// Trends server filter: 0 = all, otherwise index+1 into
    /// [`AppState::trend_servers`].
    pub trends_filter: usize,
    /// Highlighted row on the settings screen.
    pub settings_cursor: usize,
    /// A transient status message (e.g. clipboard confirmation).
    notice: Option<(String, Instant)>,
    /// Set when the visible content changed and a redraw is required.
    dirty: bool,
}

impl AppState {
    /// Creates the initial state on the splash screen.
    #[must_use]
    pub fn new(settings: Settings, theme_names: Vec<String>, provider_name: &'static str) -> Self {
        let theme_index = theme_names
            .iter()
            .position(|name| name.eq_ignore_ascii_case(&settings.theme))
            .unwrap_or(0);
        Self {
            screen: Screen::Splash,
            return_to: Screen::Splash,
            phase: None,
            ping: PingView::new(),
            download: TransferView::new(),
            upload: TransferView::new(),
            report: None,
            servers: Vec::new(),
            server_index: 0,
            server_cursor: 0,
            theme_names,
            theme_index,
            theme_cursor: theme_index,
            error: None,
            servers_loading: true,
            testing: false,
            tick: 0,
            started_at: None,
            elapsed: Duration::ZERO,
            splash_since: Instant::now(),
            settings,
            provider_name,
            client_info: None,
            preferred_server: None,
            repeat_every: None,
            finished_at: None,
            trends: Vec::new(),
            trends_filter: 0,
            settings_cursor: 0,
            notice: None,
            dirty: true,
        }
    }

    /// Unique server names present in the loaded trends, sorted.
    #[must_use]
    pub fn trend_servers(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .trends
            .iter()
            .map(|record| record.server.clone())
            .collect();
        names.sort();
        names.dedup();
        names
    }

    /// Cycles the trends server filter left or right.
    pub fn cycle_trends_filter(&mut self, delta: i64) {
        let options = self.trend_servers().len() + 1;
        if options <= 1 {
            return;
        }
        let current = self.trends_filter as i64;
        self.trends_filter = (current + delta).rem_euclid(options as i64) as usize;
        self.request_redraw();
    }

    /// The trends records passing the current server filter.
    #[must_use]
    pub fn filtered_trends(&self) -> Vec<&crate::app::history::HistoryRecord> {
        match self.trends_filter.checked_sub(1) {
            None => self.trends.iter().collect(),
            Some(index) => {
                let servers = self.trend_servers();
                match servers.get(index) {
                    Some(name) => self
                        .trends
                        .iter()
                        .filter(|record| &record.server == name)
                        .collect(),
                    None => self.trends.iter().collect(),
                }
            }
        }
    }

    /// Number of editable rows on the settings screen.
    pub const SETTINGS_ROWS: usize = 8;

    /// Adjusts the setting under the cursor by one step, within the same
    /// clamps the config loader enforces.
    pub fn adjust_setting(&mut self, delta: i64) {
        let up = delta > 0;
        let step_u64 = |value: u64, min: u64, max: u64, step: u64| -> u64 {
            if up {
                (value + step).min(max)
            } else {
                value.saturating_sub(step).max(min)
            }
        };
        match self.settings_cursor {
            0 => {
                // Theme: applies live.
                let count = self.theme_names.len().max(1);
                let current = self.theme_index as i64;
                self.theme_index = (current + delta).rem_euclid(count as i64) as usize;
                self.theme_cursor = self.theme_index;
                if let Some(name) = self.theme_names.get(self.theme_index) {
                    self.settings.theme = name.to_lowercase();
                }
            }
            1 => {
                self.settings.refresh_rate =
                    step_u64(u64::from(self.settings.refresh_rate), 5, 60, 5) as u16;
            }
            2 => {
                let value = (self.settings.animation_speed * 10.0).round() / 10.0;
                self.settings.animation_speed = if up {
                    (value + 0.1).min(5.0)
                } else {
                    (value - 0.1).max(0.1)
                };
            }
            3 => {
                self.settings.engine.ping_samples =
                    step_u64(u64::from(self.settings.engine.ping_samples), 3, 100, 1) as u32;
            }
            4 => {
                self.settings.engine.duration_secs =
                    step_u64(self.settings.engine.duration_secs, 3, 60, 1);
            }
            5 => {
                self.settings.engine.connections =
                    step_u64(self.settings.engine.connections as u64, 1, 16, 1) as usize;
            }
            6 => {
                self.settings.engine.timeout_secs =
                    step_u64(self.settings.engine.timeout_secs, 2, 120, 1);
            }
            _ => {
                self.settings.engine.upload_chunk_kb =
                    step_u64(self.settings.engine.upload_chunk_kb as u64, 64, 8192, 64) as usize;
            }
        }
        self.request_redraw();
    }

    /// Shows a transient status message for a few seconds.
    pub fn set_notice(&mut self, message: impl Into<String>) {
        self.notice = Some((message.into(), Instant::now()));
        self.request_redraw();
    }

    /// The current notice, if it has not expired yet.
    #[must_use]
    pub fn notice(&self) -> Option<&str> {
        const NOTICE_TTL: Duration = Duration::from_secs(3);
        match &self.notice {
            Some((message, shown)) if shown.elapsed() < NOTICE_TTL => Some(message),
            _ => None,
        }
    }

    /// The currently selected server, if discovery finished.
    #[must_use]
    pub fn current_server(&self) -> Option<&Server> {
        self.servers.get(self.server_index)
    }

    /// Display name of the selected server.
    #[must_use]
    pub fn server_name(&self) -> &str {
        self.current_server().map_or("—", |server| &server.name)
    }

    /// Marks the frame as needing a redraw.
    pub fn request_redraw(&mut self) {
        self.dirty = true;
    }

    /// Consumes the redraw flag for this frame.
    pub fn take_redraw(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }

    /// Clears all measurement data ahead of a new test run.
    pub fn reset_test(&mut self) {
        self.phase = None;
        self.ping.reset();
        self.download.reset();
        self.upload.reset();
        self.report = None;
        self.error = None;
        self.elapsed = Duration::ZERO;
        self.started_at = None;
        self.request_redraw();
    }

    /// Advances timers and animations by one tick.
    pub fn on_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        if self.testing {
            if let Some(started) = self.started_at {
                self.elapsed = started.elapsed();
            }
        }
        if self.screen.is_animated() {
            self.request_redraw();
        }
        // Keep the repeat countdown on the results screen ticking.
        if self.repeat_every.is_some() && self.screen == Screen::Results {
            self.request_redraw();
        }
        // Expire transient notices.
        if let Some((_, shown)) = &self.notice {
            if shown.elapsed() >= Duration::from_secs(3) {
                self.notice = None;
                self.request_redraw();
            }
        }
    }

    /// Time remaining until the next scheduled auto-restart.
    #[must_use]
    pub fn repeat_remaining(&self) -> Option<Duration> {
        let interval = self.repeat_every?;
        let finished = self.finished_at?;
        Some(interval.saturating_sub(finished.elapsed()))
    }

    /// Reduces the result of background server discovery.
    pub fn apply_servers(&mut self, result: Result<Vec<Server>, String>) {
        self.servers_loading = false;
        match result {
            Ok(servers) => {
                self.servers = servers;
                // Honor --server by preselecting the first match; fall
                // back to the nearest server when nothing matches.
                self.server_index = self
                    .preferred_server
                    .as_deref()
                    .and_then(|query| self.servers.iter().position(|s| s.matches(query)))
                    .unwrap_or(0);
                self.server_cursor = self.server_index;
            }
            Err(message) => {
                self.error = Some(message);
                self.screen = Screen::Error;
            }
        }
        self.request_redraw();
    }

    /// Reduces one engine event into the state.
    pub fn apply_engine_event(&mut self, event: EngineEvent) {
        match event {
            EngineEvent::PhaseStarted { phase } => {
                self.phase = Some(phase);
                if !self.screen.is_overlay() {
                    self.screen = Screen::Testing;
                }
            }
            EngineEvent::PingSample {
                sequence,
                latency_ms,
            } => {
                self.ping.last_ms = Some(latency_ms);
                self.ping.samples_done = sequence;
                self.ping.history.push(latency_ms);
            }
            EngineEvent::PingFinished { stats } => {
                self.ping.stats = Some(stats);
            }
            EngineEvent::Progress { progress } => {
                let view = match progress.phase {
                    TestPhase::Download => &mut self.download,
                    TestPhase::Upload => &mut self.upload,
                    TestPhase::Ping => return,
                };
                view.bytes = progress.bytes_transferred;
                view.current_bps = progress.current_bps;
                view.average_bps = progress.average_bps;
                view.peak_bps = progress.peak_bps;
                view.eta = progress.eta;
                view.ratio = progress.ratio;
                view.history.push(progress.current_bps);
            }
            EngineEvent::TransferFinished { phase, stats } => {
                let view = match phase {
                    TestPhase::Download => &mut self.download,
                    TestPhase::Upload => &mut self.upload,
                    TestPhase::Ping => return,
                };
                view.stats = Some(stats);
                view.ratio = 1.0;
                view.current_bps = stats.average_bps;
            }
            EngineEvent::Finished { report } => {
                self.report = Some(report);
                self.testing = false;
                self.phase = None;
                self.finished_at = Some(Instant::now());
                if !self.screen.is_overlay() {
                    self.screen = Screen::Results;
                }
            }
            EngineEvent::Failed { message } => {
                self.error = Some(message);
                self.testing = false;
                self.phase = None;
                self.screen = Screen::Error;
            }
        }
        self.request_redraw();
    }

    /// Marks a new test as running.
    pub fn begin_test(&mut self) {
        self.reset_test();
        self.finished_at = None;
        self.testing = true;
        self.started_at = Some(Instant::now());
        self.screen = Screen::Testing;
    }
}
