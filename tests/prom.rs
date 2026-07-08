//! Tests for Prometheus textfile output.

mod common;

use netspd::app::prom::{render, write_textfile};

#[test]
fn renders_gauges_with_escaped_labels() {
    let report = common::ReportBuilder::new()
        .server("Mumbai \"Test\"")
        .download(92.0)
        .upload(87.0)
        .ping(130.0)
        .bufferbloat(130.0, 144.0, 163.0)
        .build();

    let text = render(&report, "Ookla");
    assert!(text.contains("netspd_download_mbps"));
    assert!(text.contains("} 92\n"));
    assert!(text.contains("netspd_upload_mbps"));
    assert!(text.contains("netspd_bufferbloat_info"));
    assert!(text.contains("grade=\"B\""));
    assert!(text.contains("server=\"Mumbai \\\"Test\\\"\""));
    assert_eq!(
        text.matches("# HELP").count(),
        text.matches("# TYPE").count(),
        "every metric has HELP and TYPE"
    );
}

#[test]
fn writes_atomically_to_disk() -> Result<(), Box<dyn std::error::Error>> {
    let report = common::ReportBuilder::new()
        .download(92.0)
        .upload(87.0)
        .ping(130.0)
        .build();

    let dir = std::env::temp_dir().join(format!("netspd-prom-{}", std::process::id()));
    let path = dir.join("netspd.prom");
    write_textfile(&path, &report, "Ookla")?;
    let contents = std::fs::read_to_string(&path)?;
    assert!(contents.contains("netspd_ping_ms"));
    assert!(
        !path.with_extension("prom.tmp").exists(),
        "no temp file left"
    );
    std::fs::remove_dir_all(&dir)?;
    Ok(())
}
