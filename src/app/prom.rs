//! Prometheus node_exporter textfile output.
//!
//! With `--prom-textfile <path>` every completed run rewrites one
//! `.prom` file atomically; node_exporter's textfile collector picks it
//! up and the results land in any Prometheus/Grafana stack. Combined
//! with `--interval` this turns netspd into a permanent link monitor.

use std::io::Write;
use std::path::Path;

use crate::engine::models::TestReport;

/// One megabit, in bits.
const MBPS: f64 = 1_000_000.0;

/// Renders a report as Prometheus exposition text.
#[must_use]
pub fn render(report: &TestReport, provider: &str) -> String {
    let mut out = String::with_capacity(1024);
    let labels = format!(
        "server=\"{}\",provider=\"{}\"",
        escape(&report.server_name),
        escape(provider)
    );
    fn gauge(out: &mut String, labels: &str, name: &str, help: &str, value: f64) {
        out.push_str(&format!(
            "# HELP {name} {help}\n# TYPE {name} gauge\n{name}{{{labels}}} {value}\n"
        ));
    }

    gauge(
        &mut out,
        &labels,
        "netspd_download_mbps",
        "Average download speed of the last run.",
        report.download.average_bps / MBPS,
    );
    gauge(
        &mut out,
        &labels,
        "netspd_upload_mbps",
        "Average upload speed of the last run.",
        report.upload.average_bps / MBPS,
    );
    gauge(
        &mut out,
        &labels,
        "netspd_ping_ms",
        "Idle latency of the last run.",
        report.latency.average_ms,
    );
    gauge(
        &mut out,
        &labels,
        "netspd_jitter_ms",
        "Latency jitter of the last run.",
        report.latency.jitter_ms,
    );
    gauge(
        &mut out,
        &labels,
        "netspd_packet_loss_percent",
        "Packet loss of the last run.",
        report.latency.packet_loss_pct,
    );
    if let Some(bloat) = report.bufferbloat {
        gauge(
            &mut out,
            &labels,
            "netspd_loaded_latency_download_ms",
            "Median latency while downloading.",
            bloat.download_ms,
        );
        gauge(
            &mut out,
            &labels,
            "netspd_loaded_latency_upload_ms",
            "Median latency while uploading.",
            bloat.upload_ms,
        );
        out.push_str(&format!(
            "# HELP netspd_bufferbloat_info Bufferbloat grade of the last run.\n# TYPE netspd_bufferbloat_info gauge\nnetspd_bufferbloat_info{{{labels},grade=\"{}\"}} 1\n",
            bloat.grade.label()
        ));
    }
    gauge(
        &mut out,
        &labels,
        "netspd_last_run_timestamp_seconds",
        "Unix time of the last completed run.",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0.0, |elapsed| elapsed.as_secs_f64()),
    );
    out
}

/// Writes the exposition text atomically (write to a sibling temp file,
/// then rename), so node_exporter never reads a half-written file.
pub fn write_textfile(path: &Path, report: &TestReport, provider: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let tmp = path.with_extension("prom.tmp");
    {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(render(report, provider).as_bytes())?;
        file.sync_all()?;
    }
    std::fs::rename(&tmp, path)
}

/// Escapes a Prometheus label value.
fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
