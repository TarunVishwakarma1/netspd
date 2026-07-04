//! Tests for result history records and persistence.

use std::time::Duration;

use netspd::app::history::{append, HistoryRecord};
use netspd::engine::models::{LatencyStats, TestReport, TransferStats};

fn sample_report() -> TestReport {
    TestReport {
        server_name: "Test Server".to_owned(),
        latency: LatencyStats {
            average_ms: 12.34,
            jitter_ms: 1.29,
            min_ms: 10.0,
            max_ms: 15.0,
            samples: 10,
            packet_loss_pct: 0.0,
        },
        download: TransferStats {
            bytes: 110_000_000,
            duration: Duration::from_secs(10),
            average_bps: 88_100_000.0,
            peak_bps: 209_900_000.0,
        },
        upload: TransferStats {
            bytes: 45_000_000,
            duration: Duration::from_secs(10),
            average_bps: 44_900_000.0,
            peak_bps: 64_100_000.0,
        },
        bufferbloat: None,
    }
}

#[test]
fn record_flattens_report_into_mbps() {
    let record = HistoryRecord::from_report(&sample_report());
    assert_eq!(record.server, "Test Server");
    assert!((record.ping_ms - 12.3).abs() < 1e-9);
    assert!((record.download_mbps - 88.1).abs() < 1e-9);
    assert!((record.download_peak_mbps - 209.9).abs() < 1e-9);
    assert!((record.upload_mbps - 44.9).abs() < 1e-9);
    assert_eq!(record.download_bytes, 110_000_000);
    assert!(record.timestamp > 0);
}

#[test]
fn record_serializes_to_csv_row() {
    let record = HistoryRecord::from_report(&sample_report());
    let row = record.to_csv_row();
    assert_eq!(
        row.split(',').count(),
        HistoryRecord::CSV_HEADER.split(',').count()
    );
    assert!(row.contains("Test Server"));
    assert!(row.contains("88.1"));
}

#[test]
fn csv_quotes_fields_with_commas() {
    let mut report = sample_report();
    report.server_name = "Mumbai, India (ISP \"X\")".to_owned();
    let record = HistoryRecord::from_report(&report);
    let row = record.to_csv_row();
    assert!(row.contains("\"Mumbai, India (ISP \"\"X\"\")\""));
}

#[test]
fn record_serializes_to_json_line() -> Result<(), Box<dyn std::error::Error>> {
    let record = HistoryRecord::from_report(&sample_report());
    let json = record.to_json()?;
    assert!(!json.contains('\n'));
    let parsed: HistoryRecord = serde_json::from_str(&json)?;
    assert_eq!(parsed, record);
    Ok(())
}

#[test]
fn append_creates_directories_and_accumulates_lines() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir().join(format!("netspd-test-{}", std::process::id()));
    let path = dir.join("nested").join("history.jsonl");
    let record = HistoryRecord::from_report(&sample_report());

    append(&path, &record)?;
    append(&path, &record)?;

    let contents = std::fs::read_to_string(&path)?;
    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(lines.len(), 2);
    let parsed: HistoryRecord = serde_json::from_str(lines[1])?;
    assert_eq!(parsed.server, "Test Server");

    std::fs::remove_dir_all(&dir)?;
    Ok(())
}

#[test]
fn load_skips_bad_lines_and_honors_limit() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir().join(format!("netspd-load-{}", std::process::id()));
    let path = dir.join("history.jsonl");
    let record = HistoryRecord::from_report(&sample_report());

    netspd::app::history::append(&path, &record)?;
    // Simulate a corrupt line in the middle of the file.
    {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new().append(true).open(&path)?;
        writeln!(file, "not json at all")?;
    }
    netspd::app::history::append(&path, &record)?;
    netspd::app::history::append(&path, &record)?;

    let all = netspd::app::history::load(&path, 100);
    assert_eq!(all.len(), 3);
    let limited = netspd::app::history::load(&path, 2);
    assert_eq!(limited.len(), 2);
    // A missing file is empty, not an error.
    assert!(netspd::app::history::load(&dir.join("nope.jsonl"), 10).is_empty());

    std::fs::remove_dir_all(&dir)?;
    Ok(())
}
