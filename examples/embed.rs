//! Embedding netspd's engine in your own application.
//!
//! The engine is completely UI-free: create a provider, drive
//! [`netspd::engine::Engine`], and consume typed events. Run with:
//!
//! ```sh
//! cargo run --example embed
//! ```

use netspd::engine::models::TestPhase;
use netspd::engine::providers::{create, ProviderKind};
use netspd::engine::{Engine, EngineConfig, EngineEvent, TransferConfig};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Short phases for the example; defaults are 10 s.
    let config = EngineConfig {
        transfer: TransferConfig {
            duration: std::time::Duration::from_secs(3),
            lead_in: std::time::Duration::ZERO,
            ..TransferConfig::default()
        },
        ..EngineConfig::default()
    };

    let provider = create(ProviderKind::Librespeed, Vec::new())?;
    let engine = Engine::new(provider, config)?;

    println!("discovering servers…");
    let servers = engine.load_servers().await?;
    let server = servers
        .first()
        .ok_or_else(|| anyhow::anyhow!("no servers"))?;
    println!("testing against {}", server.name);

    let (events_tx, mut events_rx) = mpsc::channel(64);
    let cancel = CancellationToken::new();

    let run = engine.run_test(server, events_tx, cancel);
    tokio::pin!(run);

    loop {
        tokio::select! {
            maybe = events_rx.recv() => match maybe {
                Some(EngineEvent::Progress { progress })
                    if progress.phase == TestPhase::Download =>
                {
                    print!("\rdownload {:>8.1} Mbps", progress.current_bps / 1e6);
                }
                Some(EngineEvent::Finished { report }) => {
                    println!(
                        "\ndone: ↓ {:.1} Mbps  ↑ {:.1} Mbps  ping {:.0} ms",
                        report.download.average_bps / 1e6,
                        report.upload.average_bps / 1e6,
                        report.latency.average_ms
                    );
                }
                Some(_) => {}
                None => break,
            },
            result = &mut run => {
                result?;
                break;
            }
        }
    }
    Ok(())
}
