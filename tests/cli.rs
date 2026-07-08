//! Tests for command line parsing.

use clap::Parser;

use netspd::app::cli::Cli;

#[test]
fn defaults_run_the_tui() -> Result<(), clap::Error> {
    let cli = Cli::try_parse_from(["netspd"])?;
    assert!(!cli.headless());
    assert!(cli.server.is_none());
    assert!(cli.duration.is_none());
    Ok(())
}

#[test]
fn json_implies_headless() -> Result<(), clap::Error> {
    let cli = Cli::try_parse_from(["netspd", "--json"])?;
    assert!(cli.headless());
    assert!(cli.json);
    Ok(())
}

#[test]
fn list_servers_implies_headless() -> Result<(), clap::Error> {
    let cli = Cli::try_parse_from(["netspd", "--list-servers"])?;
    assert!(cli.headless());
    Ok(())
}

#[test]
fn server_flag_accepts_short_and_long() -> Result<(), clap::Error> {
    let cli = Cli::try_parse_from(["netspd", "-s", "tokyo"])?;
    assert_eq!(cli.server.as_deref(), Some("tokyo"));
    let cli = Cli::try_parse_from(["netspd", "--server", "frankfurt"])?;
    assert_eq!(cli.server.as_deref(), Some("frankfurt"));
    Ok(())
}

#[test]
fn overrides_parse_numbers() -> Result<(), clap::Error> {
    let cli = Cli::try_parse_from(["netspd", "-d", "5", "-c", "8"])?;
    assert_eq!(cli.duration, Some(5));
    assert_eq!(cli.connections, Some(8));
    Ok(())
}

#[test]
fn unknown_flags_are_rejected() {
    assert!(Cli::try_parse_from(["netspd", "--bogus"]).is_err());
}

#[test]
fn one_line_implies_headless() -> Result<(), clap::Error> {
    let cli = Cli::try_parse_from(["netspd", "--one-line"])?;
    assert!(cli.headless());
    assert!(cli.one_line);
    Ok(())
}

#[test]
fn one_line_conflicts_with_json() {
    assert!(Cli::try_parse_from(["netspd", "--one-line", "--json"]).is_err());
}

#[test]
fn one_line_conflicts_with_no_tui() {
    assert!(Cli::try_parse_from(["netspd", "--one-line", "--no-tui"]).is_err());
}
