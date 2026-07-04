//! Tests for human-friendly duration parsing.

use std::time::Duration;

use netspd::utils::duration::{parse_duration, parse_interval, MIN_INTERVAL};

#[test]
fn parses_single_units() {
    assert_eq!(parse_duration("45s").ok(), Some(Duration::from_secs(45)));
    assert_eq!(parse_duration("10m").ok(), Some(Duration::from_secs(600)));
    assert_eq!(parse_duration("2h").ok(), Some(Duration::from_secs(7200)));
}

#[test]
fn parses_compound_and_bare_seconds() {
    assert_eq!(
        parse_duration("1h30m").ok(),
        Some(Duration::from_secs(5400))
    );
    assert_eq!(parse_duration("90").ok(), Some(Duration::from_secs(90)));
    assert_eq!(parse_duration(" 5m ").ok(), Some(Duration::from_secs(300)));
}

#[test]
fn rejects_garbage_and_zero() {
    assert!(parse_duration("").is_err());
    assert!(parse_duration("fast").is_err());
    assert!(parse_duration("5x").is_err());
    assert!(parse_duration("0s").is_err());
    assert!(parse_duration("-5m").is_err());
}

#[test]
fn interval_enforces_floor() {
    assert_eq!(parse_interval("5s").ok(), Some(MIN_INTERVAL));
    assert_eq!(parse_interval("10m").ok(), Some(Duration::from_secs(600)));
}

#[test]
fn settings_expose_repeat_interval() -> Result<(), toml::de::Error> {
    let settings: netspd::config::Settings = toml::from_str("repeat_interval = \"15m\"")?;
    assert_eq!(settings.repeat_interval(), Some(Duration::from_secs(900)));
    // Typos degrade to unset, never fatal.
    let settings: netspd::config::Settings = toml::from_str("repeat_interval = \"soon\"")?;
    assert_eq!(settings.repeat_interval(), None);
    Ok(())
}
