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

    // Row 5: connections, clamped to 1..=16.
    state.settings_cursor = 5;
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
