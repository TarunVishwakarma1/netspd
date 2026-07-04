//! Headless (`--no-tui`) test runner for scripts and automation.
//!
//! Runs the same engine as the TUI, printing phase results as they
//! complete. With `--json`, the final report is emitted as a single JSON
//! object on stdout and nothing else touches stdout, so the output pipes
//! cleanly into `jq` and friends.

use anyhow::{anyhow, Context};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::engine::models::{Server, TestPhase, TestReport};
use crate::engine::{Engine, EngineEvent};
use crate::utils::format::{format_bps, format_bytes, format_millis};

use super::history::{self, HistoryRecord};

/// Buffer for engine events; matches the TUI's channel size.
const CHANNEL_CAPACITY: usize = 64;

/// How many servers are tried automatically before giving up
/// (not used when `--server` pins one explicitly).
const MAX_ATTEMPTS: usize = 3;

/// Options for a headless run.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Emit the final report as JSON on stdout.
    pub json: bool,
    /// Pick the first server whose name or host contains this text.
    pub server: Option<String>,
    /// List reachable servers and exit without testing.
    pub list_servers: bool,
}

/// Runs one complete speed test (or server listing) without a terminal
/// UI.
///
/// Human-readable progress goes to stderr; the result goes to stdout
/// (JSON when requested). Completed reports are appended to the history
/// file. Unless a server is pinned with `--server`, up to three of the
/// nearest servers are tried before failing.
pub async fn run(engine: Engine, options: Options) -> anyhow::Result<()> {
    let log = |line: String| {
        if !options.json {
            eprintln!("{line}");
        }
    };

    log(format!("netspd {} — headless", env!("CARGO_PKG_VERSION")));
    log("locating servers…".to_owned());
    let servers = engine
        .load_servers()
        .await
        .context("server discovery failed")?;

    if options.list_servers {
        for server in &servers {
            println!("{}  ({})", server.name, server.description);
        }
        return Ok(());
    }

    let candidates: Vec<Server> = match &options.server {
        Some(query) => {
            let server = servers
                .iter()
                .find(|server| server.matches(query))
                .ok_or_else(|| anyhow!("no server matching {query:?}; try --list-servers"))?;
            vec![server.clone()]
        }
        None => servers.into_iter().take(MAX_ATTEMPTS).collect(),
    };

    let total = candidates.len();
    let mut last_error = anyhow!("no servers attempted");
    for (attempt, server) in candidates.into_iter().enumerate() {
        log(format!("server: {} ({})", server.name, server.description));
        match run_once(&engine, &server, &log).await {
            Ok(report) => {
                history::record_report(&report);
                let record = HistoryRecord::from_report(&report);
                if options.json {
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
                return Ok(());
            }
            Err(err) => {
                if attempt + 1 < total {
                    log(format!("failed ({err}); trying next server…"));
                }
                last_error = err;
            }
        }
    }
    Err(last_error)
}

/// Runs one test against one server, streaming progress to `log`.
async fn run_once(
    engine: &Engine,
    server: &Server,
    log: &impl Fn(String),
) -> anyhow::Result<TestReport> {
    let (events_tx, mut events_rx) = mpsc::channel(CHANNEL_CAPACITY);
    let cancel = CancellationToken::new();
    let mut report = None;
    let mut failure = None;

    let run = engine.run_test(server, events_tx, cancel);
    tokio::pin!(run);

    loop {
        tokio::select! {
            maybe = events_rx.recv() => match maybe {
                Some(event) => note_event(event, log, &mut report, &mut failure),
                None => break,
            },
            result = &mut run => {
                result?;
                // Drain events still in flight (the final transfer and
                // completion events often land after the run future).
                while let Ok(event) = events_rx.try_recv() {
                    note_event(event, log, &mut report, &mut failure);
                }
                break;
            }
        }
    }

    if let Some(message) = failure {
        return Err(anyhow!(message));
    }
    report.ok_or_else(|| anyhow!("test produced no report"))
}

/// Logs one engine event and captures the terminal outcomes.
fn note_event(
    event: EngineEvent,
    log: &impl Fn(String),
    report: &mut Option<TestReport>,
    failure: &mut Option<String>,
) {
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
        EngineEvent::Finished { report: done } => *report = Some(done),
        EngineEvent::Failed { message } => *failure = Some(message),
        EngineEvent::PingSample { .. } | EngineEvent::Progress { .. } => {}
    }
}
