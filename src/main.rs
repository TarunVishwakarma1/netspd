//! Binary entry point: load configuration, assemble the layers, run.

use anyhow::Context;
use clap::Parser;

use netspd::app::app::App;
use netspd::app::cli::Cli;
use netspd::app::headless;
use netspd::app::state::AppState;
use netspd::config;
use netspd::engine::{providers, Engine};
use netspd::tui::renderer::Renderer;
use netspd::tui::terminal::{install_panic_hook, Tui};
use netspd::tui::theme::Theme;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let mut settings = config::load().context("failed to load configuration")?;
    settings.apply_overrides(cli.duration, cli.connections);

    let provider = providers::create(settings.provider, settings.custom_servers())
        .context("failed to initialize provider")?;
    let mut engine_config = settings.engine_config();
    if cli.headless() {
        // No UI, no ignition sweep: skip the phase lead-in entirely.
        engine_config.transfer.lead_in = std::time::Duration::ZERO;
    }
    let engine = Engine::new(provider, engine_config).context("failed to initialize engine")?;

    if cli.headless() {
        let options = headless::Options {
            json: cli.json,
            server: cli.server,
            list_servers: cli.list_servers,
        };
        return headless::run(engine, options).await;
    }

    let themes_dir = dirs::config_dir().map(|dir| dir.join("netspd").join("themes"));
    let themes = Theme::load_all(themes_dir.as_deref()).context("failed to load themes")?;

    let renderer = Renderer::new(themes, settings.animation_speed());
    let mut state = AppState::new(
        settings.clone(),
        renderer.theme_names(),
        engine.provider_name(),
    );
    state.preferred_server = cli.server;
    let app = App::new(engine, state, renderer, settings.tick_rate());

    install_panic_hook();
    let mut tui = Tui::new().context("failed to initialize terminal")?;
    let result = app.run(&mut tui).await;
    drop(tui);
    result
}
