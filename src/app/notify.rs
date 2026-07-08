//! Best-effort desktop notification fired when a test finishes.
//!
//! Uses `notify-rust` on Unix (macOS / Linux). On Windows the call is a
//! no-op — WinRT notification support would require additional platform
//! tooling that is out of scope for the current build targets.

use crate::engine::models::TestReport;
#[cfg(unix)]
use crate::utils::format::split_bps;

/// Fires a desktop notification summarising the completed test.
///
/// Errors are silently discarded: notifications are informational only and
/// must never crash or block the application.
pub fn fire(report: &TestReport, provider: &str) {
    fire_impl(report, provider);
}

// ── Unix implementation ───────────────────────────────────────────────────────

#[cfg(unix)]
fn fire_impl(report: &TestReport, provider: &str) {
    let body = build_body(report, provider);
    let _ = try_notify(&body);
}

#[cfg(unix)]
fn try_notify(body: &str) -> notify_rust::error::Result<()> {
    notify_rust::Notification::new()
        .summary("netspd — Test complete")
        .body(body)
        .appname("netspd")
        .show()?;
    Ok(())
}

// ── Windows stub ──────────────────────────────────────────────────────────────

#[cfg(not(unix))]
fn fire_impl(_report: &TestReport, _provider: &str) {}

// ── Shared helper ─────────────────────────────────────────────────────────────

#[cfg(unix)]
fn build_body(report: &TestReport, provider: &str) -> String {
    let (dl, dl_unit) = split_bps(report.download.average_bps);
    let (ul, ul_unit) = split_bps(report.upload.average_bps);
    let ping = report.latency.average_ms;
    format!("↓ {dl} {dl_unit}   ↑ {ul} {ul_unit}   ping {ping:.0} ms   via {provider}")
}
