//! The asynchronous runtime loop wiring terminal, engine and renderer.

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyEventKind};
use futures_util::StreamExt;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::engine::models::Server;
use crate::engine::{Engine, EngineConfig, EngineEvent};
use crate::tui::renderer::Renderer;
use crate::tui::terminal::Tui;

use super::action::Command;
use super::controller;
use super::event::AppEvent;
use super::history;
use super::state::{AppState, Screen};

/// Minimum time the splash screen stays visible, so startup never flashes.
const MIN_SPLASH: Duration = Duration::from_millis(1200);

/// Capacity of the engine event channel; progress events are small and
/// frequent, so a modest buffer avoids backpressure on the engine.
const ENGINE_CHANNEL_CAPACITY: usize = 64;

/// How many alternative servers are tried automatically after a failure
/// before showing the error screen.
const MAX_FAILOVER: usize = 2;

/// Number of past results loaded for the trends screen.
const TRENDS_LIMIT: usize = 120;

/// The application: owns the engine, the state and the renderer, and runs
/// the event loop until the user quits.
pub struct App {
    engine: Arc<Engine>,
    engine_config: EngineConfig,
    state: AppState,
    renderer: Renderer,
    tick_rate: Duration,
    engine_rx: Option<mpsc::Receiver<EngineEvent>>,
    servers_rx: Option<mpsc::Receiver<Result<Vec<Server>, String>>>,
    info_rx: Option<mpsc::Receiver<String>>,
    cancel: CancellationToken,
    failover_attempts: usize,
    prom_textfile: Option<std::path::PathBuf>,
    /// The screen rendered on the previous draw call; used to detect
    /// transitions that need a full terminal clear.
    last_drawn_screen: Option<Screen>,
}

impl App {
    /// Assembles the application from its parts.
    ///
    /// `engine_config` is kept so the engine can be rebuilt when the
    /// user switches providers at runtime.
    #[must_use]
    pub fn new(
        engine: Engine,
        engine_config: EngineConfig,
        state: AppState,
        renderer: Renderer,
        tick_rate: Duration,
    ) -> Self {
        Self {
            engine: Arc::new(engine),
            engine_config,
            state,
            renderer,
            tick_rate,
            engine_rx: None,
            servers_rx: None,
            info_rx: None,
            cancel: CancellationToken::new(),
            failover_attempts: 0,
            prom_textfile: None,
            last_drawn_screen: None,
        }
    }

    /// Also write Prometheus metrics after each completed run.
    #[must_use]
    pub fn with_prom_textfile(mut self, path: Option<std::path::PathBuf>) -> Self {
        self.prom_textfile = path;
        self
    }

    /// Runs the event loop until quit, drawing at most one frame per tick.
    pub async fn run(mut self, tui: &mut Tui) -> Result<()> {
        self.servers_rx = Some(self.spawn_discovery());
        let mut input = EventStream::new();
        let mut ticker = tokio::time::interval(self.tick_rate);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut last_frame = Instant::now();

        self.draw(tui, 0.0)?;

        loop {
            let event = tokio::select! {
                maybe = input.next() => match maybe {
                    Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => {
                        AppEvent::Key(key)
                    }
                    Some(Ok(Event::Mouse(mouse))) => AppEvent::Mouse(mouse.kind),
                    Some(Ok(Event::Resize(_, _))) => AppEvent::Resize,
                    Some(Ok(_)) => continue,
                    // Terminal input errored or closed; nothing sensible
                    // remains but to exit cleanly.
                    Some(Err(_)) | None => break,
                },
                _ = ticker.tick() => AppEvent::Tick,
                maybe = recv_or_pending(&mut self.engine_rx) => match maybe {
                    Some(engine_event) => AppEvent::Engine(engine_event),
                    None => {
                        self.engine_rx = None;
                        continue;
                    }
                },
                maybe = recv_or_pending(&mut self.servers_rx) => match maybe {
                    Some(result) => AppEvent::ServersLoaded(result),
                    None => {
                        self.servers_rx = None;
                        continue;
                    }
                },
                maybe = recv_or_pending(&mut self.info_rx) => match maybe {
                    Some(info) => AppEvent::ClientInfo(info),
                    None => {
                        self.info_rx = None;
                        continue;
                    }
                },
            };

            match event {
                AppEvent::Key(key) => {
                    let Some(action) = controller::map_key(self.state.screen, key) else {
                        continue;
                    };
                    let command = controller::handle(&mut self.state, action);
                    if self.execute(command) {
                        break;
                    }
                }
                AppEvent::Mouse(kind) => {
                    let Some(action) = controller::map_mouse(self.state.screen, kind) else {
                        continue;
                    };
                    let command = controller::handle(&mut self.state, action);
                    if self.execute(command) {
                        break;
                    }
                }
                AppEvent::Resize => self.state.request_redraw(),
                AppEvent::Engine(engine_event) => {
                    if matches!(engine_event, EngineEvent::Failed { .. }) && self.try_failover() {
                        continue;
                    }
                    if let EngineEvent::Finished { report } = &engine_event {
                        self.failover_attempts = 0;
                        // Best-effort persistence; the UI never blocks on it.
                        history::record_report(report);
                        if let Some(path) = &self.prom_textfile {
                            let _ =
                                super::prom::write_textfile(path, report, self.state.provider_name);
                        }
                        if self.state.settings.notify {
                            let report_clone = report.clone();
                            let provider = self.state.provider_name;
                            tokio::task::spawn_blocking(move || {
                                super::notify::fire(&report_clone, provider);
                            });
                        }
                    }
                    self.state.apply_engine_event(engine_event);
                }
                AppEvent::ServersLoaded(result) => {
                    self.state.apply_servers(result);
                    if self.info_rx.is_none() {
                        self.info_rx = self.spawn_info_fetch();
                    }
                }
                AppEvent::ClientInfo(info) => {
                    self.state.client_info = Some(info);
                    self.state.request_redraw();
                }
                AppEvent::Tick => {
                    self.state.on_tick();
                    self.maybe_leave_splash();
                    self.maybe_repeat();
                    if self.state.take_redraw() {
                        let dt = last_frame.elapsed().as_secs_f64();
                        last_frame = Instant::now();
                        self.draw(tui, dt)?;
                    }
                }
            }
        }

        self.cancel.cancel();
        Ok(())
    }

    /// Starts server discovery in the background.
    fn spawn_discovery(&self) -> mpsc::Receiver<Result<Vec<Server>, String>> {
        let (tx, rx) = mpsc::channel(1);
        let engine = Arc::clone(&self.engine);
        tokio::spawn(async move {
            let result = engine.load_servers().await.map_err(|err| err.to_string());
            let _ = tx.send(result).await;
        });
        rx
    }

    /// Fetches the client's IP/ISP through the selected server, once
    /// discovery has produced one.
    fn spawn_info_fetch(&self) -> Option<mpsc::Receiver<String>> {
        let server = self.state.current_server().cloned()?;
        let (tx, rx) = mpsc::channel(1);
        let engine = Arc::clone(&self.engine);
        tokio::spawn(async move {
            if let Some(info) = engine.client_info(&server).await {
                let _ = tx.send(info).await;
            }
        });
        Some(rx)
    }

    /// Automatically starts the first test once the splash has been shown
    /// long enough and servers are available.
    fn maybe_leave_splash(&mut self) {
        let ready = self.state.screen == Screen::Splash
            && !self.state.servers_loading
            && !self.state.testing
            && !self.state.servers.is_empty()
            && self.state.splash_since.elapsed() >= MIN_SPLASH;
        if ready {
            self.start_test();
        }
    }

    /// Performs a controller command; returns `true` when the loop
    /// should exit.
    fn execute(&mut self, command: Command) -> bool {
        match command {
            Command::Quit => return true,
            Command::StartTest => {
                self.failover_attempts = 0;
                self.start_test();
            }
            Command::LoadTrends => {
                self.state.trends = history::load_recent(TRENDS_LIMIT);
            }
            Command::Share => self.share_result(),
            Command::ReloadProvider => self.reload_provider(),
            Command::SaveConfig => match crate::config::save(&self.state.settings) {
                Ok(path) => self
                    .state
                    .set_notice(format!("saved to {}", path.display())),
                Err(err) => self.state.set_notice(err.to_string()),
            },
            Command::None => {}
        }
        false
    }

    /// Rebuilds the engine for the provider selected on the settings
    /// screen, cancels any running test and starts fresh discovery.
    fn reload_provider(&mut self) {
        let kind = self.state.settings.provider;
        let provider =
            match crate::engine::providers::create(kind, self.state.settings.custom_servers()) {
                Ok(provider) => provider,
                Err(err) => {
                    self.state.set_notice(err.to_string());
                    return;
                }
            };
        let engine = match Engine::new(provider, self.engine_config) {
            Ok(engine) => engine,
            Err(err) => {
                self.state.set_notice(err.to_string());
                return;
            }
        };

        // Stop anything in flight against the old provider.
        self.cancel.cancel();
        self.cancel = CancellationToken::new();
        self.engine_rx = None;
        self.failover_attempts = 0;

        self.engine = Arc::new(engine);
        self.state.provider_name = self.engine.provider_name();
        self.state.reset_test();
        self.state.testing = false;
        self.state.servers.clear();
        self.state.server_index = 0;
        self.state.server_cursor = 0;
        self.state.client_info = None;
        self.state.servers_loading = true;
        self.info_rx = None;
        self.servers_rx = Some(self.spawn_discovery());
        self.state
            .set_notice(format!("{} — discovering servers…", kind.label()));
    }

    /// Copies the last result to the clipboard and shows a confirmation.
    fn share_result(&mut self) {
        let Some(report) = &self.state.report else {
            return;
        };
        let text = super::share::share_text(report, self.state.provider_name);
        match super::share::copy_to_clipboard(&text) {
            Ok(()) => self.state.set_notice("✓ result copied to clipboard"),
            Err(message) => self.state.set_notice(message),
        }
    }

    /// Auto-restarts the test once the configured repeat interval has
    /// elapsed since the last completion. Only fires from the results
    /// screen, so a user browsing overlays is never yanked away.
    fn maybe_repeat(&mut self) {
        let due = self.state.screen == Screen::Results
            && !self.state.testing
            && self
                .state
                .repeat_remaining()
                .is_some_and(|remaining| remaining.is_zero());
        if due {
            self.failover_attempts = 0;
            self.start_test();
        }
    }

    /// Moves to the next-nearest server after a failure and restarts,
    /// up to [`MAX_FAILOVER`] times. Returns whether a retry started.
    ///
    /// Skipped when the user pinned a server with `--server`: an explicit
    /// choice should fail loudly, not wander.
    fn try_failover(&mut self) -> bool {
        let next = self.state.server_index + 1;
        let possible = self.state.preferred_server.is_none()
            && self.failover_attempts < MAX_FAILOVER
            && next < self.state.servers.len();
        if !possible {
            return false;
        }
        self.failover_attempts += 1;
        self.state.server_index = next;
        self.state.server_cursor = next;
        self.start_test();
        true
    }

    /// Cancels any running test and launches a new one against the
    /// selected server.
    fn start_test(&mut self) {
        let Some(server) = self.state.current_server().cloned() else {
            return;
        };
        self.cancel.cancel();
        self.cancel = CancellationToken::new();
        self.state.begin_test();

        let (tx, rx) = mpsc::channel(ENGINE_CHANNEL_CAPACITY);
        self.engine_rx = Some(rx);
        let engine = Arc::clone(&self.engine);
        let cancel = self.cancel.clone();
        tokio::spawn(async move {
            // Failures already reach the UI as EngineEvent::Failed.
            let _ = engine.run_test(&server, tx, cancel).await;
        });
    }

    /// Renders one frame.
    fn draw(&mut self, tui: &mut Tui, dt: f64) -> Result<()> {
        let current = self.state.screen;
        // After a test, the dial canvas and progress bar push many escape
        // sequences per frame. Dropped sequences leave the terminal cursor
        // position out of sync with ratatui's model, causing characters to
        // land in wrong cells on the results screen. A full clear fixes it.
        //
        // Only clear on the transition to a settled full screen (Results,
        // Error, Splash). Opening an overlay mid-test also moves `screen`
        // away from Testing, but the test is still running and the overlay
        // draws over a live Testing background — clearing there would blank-
        // flash for no benefit.
        if self.last_drawn_screen == Some(Screen::Testing)
            && current != Screen::Testing
            && !current.is_overlay()
        {
            tui.clear()?;
        }
        self.last_drawn_screen = Some(current);
        let renderer = &mut self.renderer;
        let state = &self.state;
        tui.draw(|frame| renderer.render(frame, state, dt))?;
        Ok(())
    }
}

/// Receives from an optional channel, parking forever when absent so it
/// can sit inside `tokio::select!` without busy-looping.
async fn recv_or_pending<T>(rx: &mut Option<mpsc::Receiver<T>>) -> Option<T> {
    match rx {
        Some(receiver) => receiver.recv().await,
        None => std::future::pending().await,
    }
}
