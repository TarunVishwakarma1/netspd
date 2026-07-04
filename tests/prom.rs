//! Tests for Prometheus textfile output.

use std::time::Duration;

use netspd::app::prom::{render, write_textfile};
use netspd::engine::models::{Bufferbloat, LatencyStats, TestReport, TransferStats};

fn report() -> TestReport {
    let stats = |mbps: f64| TransferStats {
        bytes: 1,
        duration: Duration::from_secs(1),
        average_bps: mbps * 1_000_000.0,
        peak_bps: mbps * 1_000_000.0,
    };
    TestReport {
        server_name: "Mumbai \"Test\"".to_owned(),
        latency: LatencyStats {
            average_ms: 130.0,
            jitter_ms: 2.0,
            min_ms: 128.0,
            max_ms: 140.0,
            samples: 10,
            packet_loss_pct: 0.0,
        },
        download: stats(92.0),
        upload: stats(87.0),
        bufferbloat: Some(Bufferbloat::new(130.0, 144.0, 163.0)),
    }
}

#[test]
fn renders_gauges_with_escaped_labels() {
    let text = render(&report(), "Ookla");
    assert!(text.contains("netspd_download_mbps"));
    assert!(text.contains("} 92\n"));
    assert!(text.contains("netspd_upload_mbps"));
    assert!(text.contains("netspd_bufferbloat_info"));
    assert!(text.contains("grade=\"B\""));
    // Quotes in server names are escaped per exposition format.
    assert!(text.contains("server=\"Mumbai \\\"Test\\\"\""));
    // Every metric declares HELP and TYPE.
    assert_eq!(
        text.matches("# HELP").count(),
        text.matches("# TYPE").count()
    );
}

#[test]
fn writes_atomically_to_disk() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir().join(format!("netspd-prom-{}", std::process::id()));
    let path = dir.join("netspd.prom");
    write_textfile(&path, &report(), "Ookla")?;
    let contents = std::fs::read_to_string(&path)?;
    assert!(contents.contains("netspd_ping_ms"));
    // No temp file left behind.
    assert!(!path.with_extension("prom.tmp").exists());
    std::fs::remove_dir_all(&dir)?;
    Ok(())
}
