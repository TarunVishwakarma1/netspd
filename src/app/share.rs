//! Shareable result cards and clipboard integration.

use std::io::Write;
use std::process::{Command, Stdio};

use crate::engine::models::TestReport;
use crate::utils::format::{format_bps, format_millis};

/// Formats a report as a compact, paste-anywhere result card.
#[must_use]
pub fn share_text(report: &TestReport, provider: &str) -> String {
    let bloat = report
        .bufferbloat
        .map(|b| format!("  bufferbloat {}", b.grade.label()))
        .unwrap_or_default();
    format!(
        "netspd — {}\n↓ {}  ↑ {}  ping {} (jitter {}, loss {:.0}%){bloat}\nvia {} · https://github.com/TarunVishwakarma1/netspd",
        report.server_name,
        format_bps(report.download.average_bps),
        format_bps(report.upload.average_bps),
        format_millis(report.latency.average_ms),
        format_millis(report.latency.jitter_ms),
        report.latency.packet_loss_pct,
        provider,
    )
}

/// The clipboard commands tried in order, per platform convention.
const CLIPBOARD_COMMANDS: [(&str, &[&str]); 4] = [
    ("pbcopy", &[]),
    ("wl-copy", &[]),
    ("xclip", &["-selection", "clipboard"]),
    ("clip.exe", &[]),
];

/// Copies text to the system clipboard through the first available
/// platform utility (`pbcopy`, `wl-copy`, `xclip`, `clip.exe`).
///
/// Returns a short human-readable error when no utility is available —
/// the UI shows it instead of failing silently.
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    for (command, args) in CLIPBOARD_COMMANDS {
        let child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        let Ok(mut child) = child else {
            continue;
        };
        let Some(stdin) = child.stdin.as_mut() else {
            continue;
        };
        if stdin.write_all(text.as_bytes()).is_err() {
            continue;
        }
        match child.wait() {
            Ok(status) if status.success() => return Ok(()),
            _ => continue,
        }
    }
    Err("no clipboard utility found (pbcopy/xclip/wl-copy)".to_owned())
}
