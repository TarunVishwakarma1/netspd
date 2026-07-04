//! Command line interface definition.

use clap::{Parser, Subcommand};

use crate::engine::providers::ProviderKind;

/// A beautiful network speed test for your terminal.
#[derive(Debug, Parser)]
#[command(name = "netspd", version, about)]
pub struct Cli {
    /// Optional mode; without one, netspd runs a speed test.
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Test against one specific server URL (a `netspd serve` instance
    /// or any LibreSpeed-compatible backend)
    #[arg(long, value_name = "URL", env = "NETSPD_URL")]
    pub url: Option<String>,

    /// Speed test provider: librespeed, ookla, fast or custom
    #[arg(long, short = 'p', value_name = "NAME", env = "NETSPD_PROVIDER")]
    pub provider: Option<ProviderKind>,

    /// Run headless: progress on stderr, summary on stdout
    #[arg(long)]
    pub no_tui: bool,

    /// Run headless and print the report as one JSON object on stdout
    #[arg(long, conflicts_with = "csv", env = "NETSPD_JSON")]
    pub json: bool,

    /// Run headless and print the report as CSV on stdout
    /// (header + one row per run)
    #[arg(long)]
    pub csv: bool,

    /// Pick a server whose name or host contains this text
    /// (case-insensitive)
    #[arg(long, short = 's', value_name = "TEXT", env = "NETSPD_SERVER")]
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

    /// Repeat the test on an interval, e.g. 45s, 10m, 2h (min 30s).
    /// Headless mode loops forever; the TUI auto-restarts after results
    #[arg(long, short = 'i', value_name = "DURATION", env = "NETSPD_INTERVAL")]
    pub interval: Option<String>,

    /// Plain ASCII UI: no braille, block art or unicode symbols
    #[arg(long)]
    pub ascii: bool,

    /// Test the N nearest servers and print a ranked comparison
    #[arg(long, value_name = "N", conflicts_with_all = ["server", "interval"])]
    pub compare: Option<usize>,

    /// Print stored results and exit (respects --json / --csv)
    #[arg(long)]
    pub history: bool,

    /// Force IPv4 for measurements
    #[arg(short = '4', long = "ipv4", conflicts_with = "ipv6")]
    pub ipv4: bool,

    /// Force IPv6 for measurements
    #[arg(short = '6', long = "ipv6")]
    pub ipv6: bool,

    /// Exit with code 2 when download falls below this many Mbps
    /// (for CI checks and alerting)
    #[arg(long, value_name = "MBPS", env = "NETSPD_FAIL_BELOW")]
    pub fail_below: Option<f64>,

    /// Write Prometheus metrics to this node_exporter textfile after
    /// each completed run
    #[arg(long, value_name = "PATH", env = "NETSPD_PROM_TEXTFILE")]
    pub prom_textfile: Option<std::path::PathBuf>,
}

/// Alternate modes.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run the built-in speed test server (the target for `--url`),
    /// e.g. inside a pod so other pods can measure the link to it
    Serve {
        /// Port to listen on
        #[arg(long, default_value_t = crate::app::serve::DEFAULT_PORT, env = "NETSPD_PORT")]
        port: u16,
        /// Address to bind
        #[arg(long, default_value = "0.0.0.0", env = "NETSPD_BIND")]
        bind: String,
    },
    /// Print shell completions to stdout
    Completions {
        /// Target shell
        shell: clap_complete::Shell,
    },
    /// Print the man page (roff) to stdout
    Man,
}

/// Prints completions for `shell`.
pub fn print_completions(shell: clap_complete::Shell) {
    use clap::CommandFactory;
    let mut command = Cli::command();
    clap_complete::generate(shell, &mut command, "netspd", &mut std::io::stdout());
}

/// Prints the roff man page.
pub fn print_man() -> std::io::Result<()> {
    use clap::CommandFactory;
    let man = clap_mangen::Man::new(Cli::command());
    man.render(&mut std::io::stdout())
}

impl Cli {
    /// Whether any flag requests a run without the TUI.
    #[must_use]
    pub fn headless(&self) -> bool {
        self.no_tui
            || self.json
            || self.csv
            || self.list_servers
            || self.compare.is_some()
            || self.history
    }
}
