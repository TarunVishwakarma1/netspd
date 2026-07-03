//! Headless (`--no-tui`) test runner for scripts and automation.
//!
//! Runs the same engine as the TUI, printing phase results as they
//! complete. With `--json`, the final report is emitted as a single JSON
//! object on stdout and nothing else touches stdout, so the output pipes
//! cleanly into `jq` and friends.

use anyhow::{anyhow, Context};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::engine::models::TestPhase;
use crate::engine::{Engine, EngineEvent};
use crate::utils::format::{format_bps, format_bytes, format_millis};

use super::history::{self, HistoryRecord};

/// Buffer for engine events; matches the TUI's channel size.
const CHANNEL_CAPACITY: usize = 64;

/// Runs one complete speed test without a terminal UI.
///
/// Human-readable progress goes to stderr; the result goes to stdout
/// (JSON when `json` is set). The report is appended to the history file
/// either way.
pub async fn run(engine: Engine, json: bool) -> anyhow::Result<()> {
    let log = |line: String| {
        if !json {
            eprintln!("{line}");
        }
    };

    log(format!("netspd {} — headless", env!("CARGO_PKG_VERSION")));
    log("locating servers…".to_owned());
    let servers = engine
        .load_servers()
        .await
        .context("server discovery failed")?;
    let server = servers
        .first()
        .ok_or_else(|| anyhow!("no reachable servers"))?
        .clone();
    log(format!("server: {} ({})", server.name, server.description));

    let (events_tx, mut events_rx) = mpsc::channel(CHANNEL_CAPACITY);
    let cancel = CancellationToken::new();
    let test_cancel = cancel.clone();
    let runner =
        tokio::spawn(async move { engine.run_test(&server, events_tx, test_cancel).await });

    let mut report = None;
    while let Some(event) = events_rx.recv().await {
        match event {
            EngineEvent::PhaseStarted { phase } => {
                log(format!("{}…", phase.label().to_lowercase()));
            }
            EngineEvent::PingFinished { stats } => {
                log(format!(
                    "  ping {}  jitter {}  loss {:.0}%",
                    format_millis(stats.average_ms),
                    format_millis(stats.jitter_ms),
                    stats.packet_loss_pct
                ));
            }
            EngineEvent::TransferFinished { phase, stats } => {
                let arrow = match phase {
                    TestPhase::Download => "↓",
                    TestPhase::Upload => "↑",
                    TestPhase::Ping => " ",
                };
                log(format!(
                    "  {arrow} {}  peak {}  ({})",
                    format_bps(stats.average_bps),
                    format_bps(stats.peak_bps),
                    format_bytes(stats.bytes)
                ));
            }
            EngineEvent::Finished { report: done } => report = Some(done),
            EngineEvent::Failed { message } => return Err(anyhow!(message)),
            EngineEvent::PingSample { .. } | EngineEvent::Progress { .. } => {}
        }
    }
    runner.await.context("test task failed")??;

    let report = report.ok_or_else(|| anyhow!("test produced no report"))?;
    history::record_report(&report);

    let record = HistoryRecord::from_report(&report);
    if json {
        println!("{}", record.to_json().context("failed to encode report")?);
    } else {
        println!(
            "{}: ↓ {} ↑ {} ping {}",
            report.server_name,
            format_bps(report.download.average_bps),
            format_bps(report.upload.average_bps),
            format_millis(report.latency.average_ms),
        );
    }
    Ok(())
}
