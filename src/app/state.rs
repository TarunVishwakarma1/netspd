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
    /// A fatal test error.
    Error,
}

impl Screen {
    /// Whether this screen is an overlay that returns to a parent screen.
    #[must_use]
    pub fn is_overlay(self) -> bool {
        matches!(
            self,
            Self::Help | Self::Settings | Self::ServerSelect | Self::ThemeSelect
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
            dirty: true,
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
    }

    /// Reduces the result of background server discovery.
    pub fn apply_servers(&mut self, result: Result<Vec<Server>, String>) {
        self.servers_loading = false;
        match result {
            Ok(servers) => {
                self.servers = servers;
                self.server_index = 0;
                self.server_cursor = 0;
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
        self.testing = true;
        self.started_at = Some(Instant::now());
        self.screen = Screen::Testing;
    }
}
