//! Binary entry point: load configuration, assemble the layers, run.

use anyhow::Context;

use netspd::app::app::App;
use netspd::app::headless;
use netspd::app::state::AppState;
use netspd::config;
use netspd::engine::{providers, Engine};
use netspd::tui::renderer::Renderer;
use netspd::tui::terminal::{install_panic_hook, Tui};
use netspd::tui::theme::Theme;

/// Parsed command line flags.
struct CliArgs {
    /// Run without the terminal UI.
    no_tui: bool,
    /// Emit the final report as JSON (implies `no_tui`).
    json: bool,
}

const USAGE: &str = "\
netspd — network speed testing in the terminal

USAGE:
    netspd [OPTIONS]

OPTIONS:
    --no-tui     run headless, printing progress to stderr
    --json       run headless and print the report as JSON on stdout
    -h, --help   show this help
    -V, --version  show the version";

/// Parses arguments by hand; the surface is small enough that a
/// dependency would outweigh it.
fn parse_args() -> anyhow::Result<Option<CliArgs>> {
    let mut args = CliArgs {
        no_tui: false,
        json: false,
    };
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--no-tui" => args.no_tui = true,
            "--json" => {
                args.json = true;
                args.no_tui = true;
            }
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(None);
            }
            "-V" | "--version" => {
                println!("netspd {}", env!("CARGO_PKG_VERSION"));
                return Ok(None);
            }
            other => anyhow::bail!("unknown option {other:?}; see --help"),
        }
    }
    Ok(Some(args))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Some(args) = parse_args()? else {
        return Ok(());
    };

    let settings = config::load().context("failed to load configuration")?;
    let provider = providers::create(settings.provider, settings.custom_servers())
        .context("failed to initialize provider")?;
    let mut engine_config = settings.engine_config();
    if args.no_tui {
        // No UI, no ignition sweep: skip the phase lead-in entirely.
        engine_config.transfer.lead_in = std::time::Duration::ZERO;
    }
    let engine = Engine::new(provider, engine_config).context("failed to initialize engine")?;

    if args.no_tui {
        return headless::run(engine, args.json).await;
    }

    let themes_dir = dirs::config_dir().map(|dir| dir.join("netspd").join("themes"));
    let themes = Theme::load_all(themes_dir.as_deref()).context("failed to load themes")?;

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
