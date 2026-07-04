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
    let mut cli = Cli::parse();

    if let Some(netspd::app::cli::Commands::Serve { port, bind }) = &cli.command {
        return netspd::app::serve::run(bind, *port).await;
    }

    // Inside containers and pipelines there is no terminal to draw on;
    // default to headless instead of failing.
    if !cli.headless() && !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        cli.no_tui = true;
    }

    let mut settings = config::load().context("failed to load configuration")?;
    settings.apply_overrides(cli.duration, cli.connections);
    if let Some(provider) = cli.provider {
        settings.provider = provider;
    }

    // --url pins one ad-hoc server (a `netspd serve` peer or any
    // LibreSpeed-compatible backend) without touching the config file.
    let custom_servers = match &cli.url {
        Some(url) => {
            settings.provider = netspd::engine::providers::ProviderKind::Custom;
            vec![netspd::engine::models::Server::from_base(
                url.trim_end_matches('/'),
                url,
                "download?bytes=26214400",
                "upload",
                "ping",
            )]
        }
        None => settings.custom_servers(),
    };
    let repeat = match cli.interval.as_deref() {
        Some(value) => Some(
            netspd::utils::duration::parse_interval(value)
                .map_err(|err| anyhow::anyhow!("--interval: {err}"))?,
        ),
        None => settings.repeat_interval(),
    };

    let provider = providers::create(settings.provider, custom_servers)
        .context("failed to initialize provider")?;
    let mut engine_config = settings.engine_config();
    if cli.headless() {
        // No UI, no ignition sweep: skip the phase lead-in entirely.
        engine_config.transfer.lead_in = std::time::Duration::ZERO;
    }
    engine_config.ip_family = if cli.ipv4 {
        Some(netspd::engine::IpFamily::V4)
    } else if cli.ipv6 {
        Some(netspd::engine::IpFamily::V6)
    } else {
        None
    };
    let engine = Engine::new(provider, engine_config).context("failed to initialize engine")?;

    if cli.history {
        headless::print_history(cli.json, cli.csv);
        return Ok(());
    }

    if cli.headless() {
        let options = headless::Options {
            json: cli.json,
            csv: cli.csv,
            server: cli.server,
            list_servers: cli.list_servers,
            interval: repeat,
            compare: cli.compare,
        };
        return headless::run(engine, options).await;
    }

    netspd::tui::glyphs::select(cli.ascii);

    let themes_dir = dirs::config_dir().map(|dir| dir.join("netspd").join("themes"));
    let mut themes = Theme::load_all(themes_dir.as_deref()).context("failed to load themes")?;
    // https://no-color.org: any non-empty NO_COLOR disables all colors.
    if std::env::var("NO_COLOR").is_ok_and(|value| !value.is_empty()) {
        themes = themes.iter().map(Theme::without_colors).collect();
    }

    let renderer = Renderer::new(themes, settings.animation_speed());
    let mut state = AppState::new(
        settings.clone(),
        renderer.theme_names(),
        engine.provider_name(),
    );
    state.preferred_server = cli.server;
    state.repeat_every = repeat;
    let app = App::new(engine, state, renderer, settings.tick_rate());

    install_panic_hook();
    let mut tui = Tui::new().context("failed to initialize terminal")?;
    let result = app.run(&mut tui).await;
    drop(tui);
    result
}
