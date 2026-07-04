//! Tests for state-side UI logic: trends filtering and settings editing.

use netspd::app::history::HistoryRecord;
use netspd::app::state::AppState;
use netspd::config::Settings;

fn record(server: &str, down: f64) -> HistoryRecord {
    HistoryRecord {
        timestamp: 1,
        server: server.to_owned(),
        ping_ms: 10.0,
        jitter_ms: 1.0,
        packet_loss_pct: 0.0,
        download_mbps: down,
        download_peak_mbps: down,
        download_bytes: 1,
        upload_mbps: down / 2.0,
        upload_peak_mbps: down,
        upload_bytes: 1,
        loaded_down_ms: None,
        loaded_up_ms: None,
        bufferbloat: None,
    }
}

fn state_with_trends() -> AppState {
    let mut state = AppState::new(
        Settings::default(),
        vec!["Default".to_owned(), "Nord".to_owned()],
        "LibreSpeed",
    );
    state.trends = vec![
        record("Alpha", 10.0),
        record("Beta", 20.0),
        record("Alpha", 30.0),
    ];
    state
}

#[test]
fn trends_filter_cycles_and_filters() {
    let mut state = state_with_trends();
    assert_eq!(state.trend_servers(), vec!["Alpha", "Beta"]);
    assert_eq!(state.filtered_trends().len(), 3);

    state.cycle_trends_filter(1); // Alpha
    assert_eq!(state.filtered_trends().len(), 2);
    state.cycle_trends_filter(1); // Beta
    assert_eq!(state.filtered_trends().len(), 1);
    state.cycle_trends_filter(1); // wraps to all
    assert_eq!(state.filtered_trends().len(), 3);
    state.cycle_trends_filter(-1); // back to Beta
    assert_eq!(state.filtered_trends().len(), 1);
}

#[test]
fn settings_adjust_respects_clamps() {
    let mut state = state_with_trends();

    // Row 6: connections, clamped to 1..=16.
    state.settings_cursor = 6;
    for _ in 0..40 {
        state.adjust_setting(1);
    }
    assert_eq!(state.settings.engine.connections, 16);
    for _ in 0..40 {
        state.adjust_setting(-1);
    }
    assert_eq!(state.settings.engine.connections, 1);

    // Row 0: theme cycles and applies live.
    state.settings_cursor = 0;
    let before = state.theme_index;
    state.adjust_setting(1);
    assert_ne!(state.theme_index, before);
    assert_eq!(
        state.settings.theme,
        state.theme_names[state.theme_index].to_lowercase()
    );
}

#[test]
fn settings_provider_row_cycles_and_signals_reload() {
    use netspd::engine::providers::ProviderKind;

    let mut state = state_with_trends();
    state.settings_cursor = 1;

    // No [[servers]] configured: custom is not offered.
    assert_eq!(state.provider_choices().len(), 3);

    assert_eq!(state.settings.provider, ProviderKind::Librespeed);
    assert!(state.adjust_setting(1)); // → Ookla, needs engine rebuild
    assert_eq!(state.settings.provider, ProviderKind::Ookla);
    assert!(state.adjust_setting(1)); // → Fast
    assert_eq!(state.settings.provider, ProviderKind::Fast);
    assert!(state.adjust_setting(1)); // wraps → Librespeed
    assert_eq!(state.settings.provider, ProviderKind::Librespeed);

    // Non-provider rows never signal a reload.
    state.settings_cursor = 0;
    assert!(!state.adjust_setting(1));
}

#[test]
fn settings_repeat_row_cycles_presets_live() {
    let mut state = state_with_trends();
    state.settings_cursor = netspd::app::state::AppState::SETTINGS_ROWS - 1;

    assert!(state.repeat_every.is_none());
    assert!(!state.adjust_setting(1)); // off → 30s
    assert_eq!(state.repeat_every, Some(std::time::Duration::from_secs(30)));
    assert_eq!(state.settings.repeat_interval.as_deref(), Some("30s"));
    state.adjust_setting(-1); // back to off
    assert!(state.repeat_every.is_none());
    assert!(state.settings.repeat_interval.is_none());
}
