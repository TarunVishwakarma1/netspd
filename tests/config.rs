//! Tests for configuration parsing and its mapping onto engine settings.

use std::time::Duration;

use netspd::config::Settings;

#[test]
fn defaults_are_sane() {
    let settings = Settings::default();
    assert_eq!(settings.theme, "default");
    assert_eq!(settings.refresh_rate, 30);
    assert!(settings.servers.is_empty());
    assert_eq!(settings.tick_rate(), Duration::from_millis(1000 / 30));
}

#[test]
fn empty_toml_yields_defaults() -> Result<(), toml::de::Error> {
    let settings: Settings = toml::from_str("")?;
    assert_eq!(settings.refresh_rate, Settings::default().refresh_rate);
    Ok(())
}

#[test]
fn full_config_parses() -> Result<(), toml::de::Error> {
    let source = r#"
        theme = "nord"
        refresh_rate = 60
        provider = "librespeed"
        animation_speed = 2.0

        [engine]
        ping_samples = 20
        ping_interval_ms = 50
        duration_secs = 5
        connections = 8
        timeout_secs = 10
        upload_chunk_kb = 1024

        [[servers]]
        name = "Local"
        url = "https://speed.example.com/backend/"
    "#;
    let settings: Settings = toml::from_str(source)?;
    assert_eq!(settings.theme, "nord");
    assert_eq!(settings.refresh_rate, 60);
    assert!((settings.animation_speed() - 2.0).abs() < f64::EPSILON);

    let engine = settings.engine_config();
    assert_eq!(engine.ping.samples, 20);
    assert_eq!(engine.ping.interval, Duration::from_millis(50));
    assert_eq!(engine.transfer.duration, Duration::from_secs(5));
    assert_eq!(engine.transfer.connections, 8);
    assert_eq!(engine.transfer.upload_chunk_bytes, 1024 * 1024);

    let servers = settings.custom_servers();
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].name, "Local");
    assert_eq!(
        servers[0].endpoints.ping,
        "https://speed.example.com/backend/empty.php"
    );
    Ok(())
}

#[test]
fn out_of_range_values_are_clamped() -> Result<(), toml::de::Error> {
    let source = r#"
        [engine]
        ping_samples = 1000
        duration_secs = 999
        connections = 200
    "#;
    let settings: Settings = toml::from_str(source)?;
    let engine = settings.engine_config();
    assert_eq!(engine.ping.samples, 100);
    assert_eq!(engine.transfer.duration, Duration::from_secs(60));
    assert_eq!(engine.transfer.connections, 16);
    Ok(())
}

#[test]
fn unknown_keys_are_rejected() {
    let result: Result<Settings, _> = toml::from_str("not_a_real_key = true");
    assert!(result.is_err());
}

#[test]
fn invalid_animation_speed_falls_back() -> Result<(), toml::de::Error> {
    let settings: Settings = toml::from_str("animation_speed = inf")?;
    assert!((settings.animation_speed() - 1.0).abs() < f64::EPSILON);
    Ok(())
}

#[test]
fn blank_custom_servers_are_ignored() -> Result<(), toml::de::Error> {
    let source = r#"
        [[servers]]
        name = "  "
        url = "https://speed.example.com/"
    "#;
    let settings: Settings = toml::from_str(source)?;
    assert!(settings.custom_servers().is_empty());
    Ok(())
}
