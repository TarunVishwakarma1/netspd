//! Headless (`--no-tui`) test runner for scripts and automation.
//!
//! Runs the same engine as the TUI, printing phase results as they
//! complete. With `--json`, the final report is emitted as a single JSON
//! object on stdout and nothing else touches stdout, so the output pipes
//! cleanly into `jq` and friends.

use anyhow::{anyhow, Context};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::app::score::CompositeScore;
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
    /// Emit the final report as CSV on stdout (header then rows).
    pub csv: bool,
    /// Pick the first server whose name or host contains this text.
    pub server: Option<String>,
    /// List reachable servers and exit without testing.
    pub list_servers: bool,
    /// Repeat the test on this interval (start to start), forever.
    pub interval: Option<std::time::Duration>,
    /// Test the N nearest servers and print a ranked comparison.
    pub compare: Option<usize>,
    /// Exit with code 2 when download falls below this many Mbps.
    pub fail_below: Option<f64>,
    /// Write Prometheus metrics here after each completed run.
    pub prom_textfile: Option<std::path::PathBuf>,
    /// Emit one compact line on stdout (for tmux / status bars).
    pub one_line: bool,
}

/// Runs headless: a single test, a server listing, or — with an
/// interval — an endless watch loop.
///
/// Human-readable progress goes to stderr; results go to stdout (JSON
/// Lines when requested). Completed reports are appended to the history
/// file. Unless a server is pinned with `--server`, up to three of the
/// nearest servers are tried per run. In watch mode a failed run is
/// logged and the loop continues — transient outages are exactly what
/// scheduled testing is for.
pub async fn run(engine: Engine, options: Options) -> anyhow::Result<()> {
    let log = |line: String| {
        if !options.json && !options.csv && !options.one_line {
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

    if options.csv {
        println!("{}", HistoryRecord::CSV_HEADER);
    }

    if let Some(count) = options.compare {
        return compare_servers(&engine, &servers, count.clamp(2, 8), &options, &log).await;
    }

    let Some(interval) = options.interval else {
        return test_once(&engine, &servers, &options, &log).await;
    };

    log(format!(
        "repeating every {} (Ctrl+C to stop)",
        crate::utils::format::format_eta(interval)
    ));
    loop {
        let started = std::time::Instant::now();
        if let Err(err) = test_once(&engine, &servers, &options, &log).await {
            log(format!("run failed ({err}); continuing"));
        }
        let wait = interval.saturating_sub(started.elapsed());
        log(format!(
            "next run in {}",
            crate::utils::format::format_eta(wait)
        ));
        tokio::time::sleep(wait).await;
    }
}

/// Tests the `count` nearest servers back to back and prints a ranked
/// comparison (best download first). Individual failures are noted and
/// skipped, not fatal — unless every server fails.
async fn compare_servers(
    engine: &Engine,
    servers: &[Server],
    count: usize,
    options: &Options,
    log: &impl Fn(String),
) -> anyhow::Result<()> {
    let mut results: Vec<HistoryRecord> = Vec::new();
    for server in servers.iter().take(count) {
        log(format!("server: {} ({})", server.name, server.description));
        match run_once(engine, server, log).await {
            Ok(report) => {
                history::record_report(&report);
                results.push(HistoryRecord::from_report(&report));
            }
            Err(err) => log(format!("  skipped ({err})")),
        }
    }
    if results.is_empty() {
        return Err(anyhow!("every compared server failed"));
    }
    results.sort_by(|a, b| b.download_mbps.total_cmp(&a.download_mbps));

    if options.json {
        println!(
            "{}",
            serde_json::to_string(&results).context("failed to encode comparison")?
        );
    } else if options.csv {
        for record in &results {
            println!("{}", record.to_csv_row());
        }
    } else {
        println!(
            "{rank:<3} {server:<40} {ping:>9} {down:>12} {up:>12}  bloat",
            rank = "#",
            server = "server",
            ping = "ping",
            down = "down",
            up = "up",
        );
        for (rank, record) in results.iter().enumerate() {
            println!(
                "{:<3} {:<40} {:>9} {:>12} {:>12}  {}",
                rank + 1,
                record.server.chars().take(40).collect::<String>(),
                format_millis(record.ping_ms),
                format!("{} Mbps", record.download_mbps),
                format!("{} Mbps", record.upload_mbps),
                record.bufferbloat.as_deref().unwrap_or("-"),
            );
        }
    }
    Ok(())
}

/// Prints stored history (newest last) in the chosen format and returns.
pub fn print_history(json: bool, csv: bool) {
    let records = history::load_recent(usize::MAX);
    if csv {
        println!("{}", HistoryRecord::CSV_HEADER);
    }
    for record in &records {
        if json {
            if let Ok(line) = record.to_json() {
                println!("{line}");
            }
        } else if csv {
            println!("{}", record.to_csv_row());
        } else {
            println!(
                "{}  {}: v {} Mbps  ^ {} Mbps  ping {}",
                record.timestamp,
                record.server,
                record.download_mbps,
                record.upload_mbps,
                format_millis(record.ping_ms),
            );
        }
    }
    if records.is_empty() && !json && !csv {
        eprintln!("no stored results yet — run a test first");
    }
}

/// Runs one test with failover across the nearest servers, printing the
/// result to stdout.
async fn test_once(
    engine: &Engine,
    servers: &[Server],
    options: &Options,
    log: &impl Fn(String),
) -> anyhow::Result<()> {
    let candidates: Vec<Server> = match &options.server {
        Some(query) => {
            let server = servers
                .iter()
                .find(|server| server.matches(query))
                .ok_or_else(|| anyhow!("no server matching {query:?}; try --list-servers"))?;
            vec![server.clone()]
        }
        None => servers.iter().take(MAX_ATTEMPTS).cloned().collect(),
    };

    let total = candidates.len();
    let mut last_error = anyhow!("no servers attempted");
    for (attempt, server) in candidates.into_iter().enumerate() {
        log(format!("server: {} ({})", server.name, server.description));
        match run_once(engine, &server, log).await {
            Ok(report) => {
                history::record_report(&report);
                if let Some(path) = &options.prom_textfile {
                    if let Err(err) =
                        super::prom::write_textfile(path, &report, engine.provider_name())
                    {
                        log(format!("failed to write {}: {err}", path.display()));
                    }
                }
                let record = HistoryRecord::from_report(&report);
                if options.json {
                    println!("{}", record.to_json().context("failed to encode report")?);
                } else if options.csv {
                    println!("{}", record.to_csv_row());
                } else if options.one_line {
                    println!("{}", format_one_line(&report));
                } else {
                    println!(
                        "{}: ↓ {} ↑ {} ping {}",
                        report.server_name,
                        format_bps(report.download.average_bps),
                        format_bps(report.upload.average_bps),
                        format_millis(report.latency.average_ms),
                    );
                    println!("{}", super::verdict::verdict(&report));
                }
                if let Some(threshold) = options.fail_below {
                    let mbps = report.download.average_bps / 1_000_000.0;
                    if mbps < threshold {
                        // Distinct exit code for alerting: 1 = test
                        // failed, 2 = test ran but missed the threshold.
                        eprintln!("download {mbps:.1} Mbps below threshold {threshold} Mbps");
                        std::process::exit(2);
                    }
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

/// Formats a completed report as a single compact line for tmux/status bars.
///
/// Example output: `↓93.7 ↑66.1 ~12ms A+`
///
/// Fields: download Mbps · upload Mbps · ping with `~` prefix · score grade.
fn format_one_line(report: &TestReport) -> String {
    let dl = report.download.average_bps / 1_000_000.0;
    let ul = report.upload.average_bps / 1_000_000.0;
    let ping = report.latency.average_ms.round() as u64;
    let grade = CompositeScore::compute(report).grade.label();
    format!("↓{dl:.1} ↑{ul:.1} ~{ping}ms {grade}")
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
        EngineEvent::Finished { report: done } => {
            if let Some(bloat) = done.bufferbloat {
                log(format!(
                    "  bufferbloat {}  (idle {} → down {} / up {})",
                    bloat.grade.label(),
                    format_millis(bloat.idle_ms),
                    format_millis(bloat.download_ms),
                    format_millis(bloat.upload_ms)
                ));
            }
            *report = Some(done);
        }
        EngineEvent::Failed { message } => *failure = Some(message),
        EngineEvent::PingSample { .. } | EngineEvent::Progress { .. } => {}
    }
}
