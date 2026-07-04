//! Command line interface definition.

use clap::Parser;

/// A beautiful network speed test for your terminal.
#[derive(Debug, Parser)]
#[command(name = "netspd", version, about)]
pub struct Cli {
    /// Run headless: progress on stderr, summary on stdout
    #[arg(long)]
    pub no_tui: bool,

    /// Run headless and print the report as one JSON object on stdout
    #[arg(long)]
    pub json: bool,

    /// Pick a server whose name or host contains this text
    /// (case-insensitive)
    #[arg(long, short = 's', value_name = "TEXT")]
    pub server: Option<String>,

    /// List reachable servers (nearest first) and exit
    #[arg(long)]
    pub list_servers: bool,

    /// Override the duration of each transfer phase, in seconds (3-60)
    #[arg(long, short = 'd', value_name = "SECS")]
    pub duration: Option<u64>,

    /// Override the number of parallel connections (1-16)
    #[arg(long, short = 'c', value_name = "N")]
    pub connections: Option<usize>,
}

impl Cli {
    /// Whether any flag requests a run without the TUI.
    #[must_use]
    pub fn headless(&self) -> bool {
        self.no_tui || self.json || self.list_servers
    }
}
