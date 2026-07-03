//! Binary entry point: load configuration, assemble the layers, run.

use anyhow::Context;

use netspd::app::app::App;
use netspd::app::state::AppState;
use netspd::config;
use netspd::engine::{providers, Engine};
use netspd::tui::renderer::Renderer;
use netspd::tui::terminal::{install_panic_hook, Tui};
use netspd::tui::theme::Theme;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = config::load().context("failed to load configuration")?;

    let themes_dir = dirs::config_dir().map(|dir| dir.join("netspd").join("themes"));
    let themes = Theme::load_all(themes_dir.as_deref()).context("failed to load themes")?;

    let provider = providers::create(settings.provider, settings.custom_servers())
        .context("failed to initialize provider")?;
    let engine =
        Engine::new(provider, settings.engine_config()).context("failed to initialize engine")?;

    let renderer = Renderer::new(themes, settings.animation_speed());
    let state = AppState::new(
        settings.clone(),
        renderer.theme_names(),
        engine.provider_name(),
    );
    let app = App::new(engine, state, renderer, settings.tick_rate());

    install_panic_hook();
    let mut tui = Tui::new().context("failed to initialize terminal")?;
    let result = app.run(&mut tui).await;
    drop(tui);
    result
}
