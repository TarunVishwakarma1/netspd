//! The asynchronous runtime loop wiring terminal, engine and renderer.

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyEventKind};
use futures_util::StreamExt;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::engine::models::Server;
use crate::engine::{Engine, EngineEvent};
use crate::tui::renderer::Renderer;
use crate::tui::terminal::Tui;

use super::action::Command;
use super::controller;
use super::event::AppEvent;
use super::state::{AppState, Screen};

/// Minimum time the splash screen stays visible, so startup never flashes.
const MIN_SPLASH: Duration = Duration::from_millis(1200);

/// Capacity of the engine event channel; progress events are small and
/// frequent, so a modest buffer avoids backpressure on the engine.
const ENGINE_CHANNEL_CAPACITY: usize = 64;

/// The application: owns the engine, the state and the renderer, and runs
/// the event loop until the user quits.
pub struct App {
    engine: Arc<Engine>,
    state: AppState,
    renderer: Renderer,
    tick_rate: Duration,
    engine_rx: Option<mpsc::Receiver<EngineEvent>>,
    cancel: CancellationToken,
}

impl App {
    /// Assembles the application from its parts.
    #[must_use]
    pub fn new(engine: Engine, state: AppState, renderer: Renderer, tick_rate: Duration) -> Self {
        Self {
            engine: Arc::new(engine),
            state,
            renderer,
            tick_rate,
            engine_rx: None,
            cancel: CancellationToken::new(),
        }
    }

    /// Runs the event loop until quit, drawing at most one frame per tick.
    pub async fn run(mut self, tui: &mut Tui) -> Result<()> {
        let mut servers_rx = Some(self.spawn_discovery());
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
                maybe = recv_or_pending(&mut servers_rx) => match maybe {
                    Some(result) => AppEvent::ServersLoaded(result),
                    None => {
                        servers_rx = None;
                        continue;
                    }
                },
            };

            match event {
                AppEvent::Key(key) => {
                    let Some(action) = controller::map_key(self.state.screen, key) else {
                        continue;
                    };
                    match controller::handle(&mut self.state, action) {
                        Command::Quit => break,
                        Command::StartTest => self.start_test(),
                        Command::None => {}
                    }
                }
                AppEvent::Resize => self.state.request_redraw(),
                AppEvent::Engine(engine_event) => self.state.apply_engine_event(engine_event),
                AppEvent::ServersLoaded(result) => self.state.apply_servers(result),
                AppEvent::Tick => {
                    self.state.on_tick();
                    self.maybe_leave_splash();
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
