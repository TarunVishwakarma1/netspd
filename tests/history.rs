//! Tests for result history records and persistence.

mod common;

use netspd::app::history::{append, HistoryRecord};

#[test]
fn record_flattens_report_into_mbps() {
    let report = common::ReportBuilder::new()
        .server("Test Server")
        .download(88.1)
        .upload(44.9)
        .ping(12.34)
        .jitter(1.29)
        .build();

    let record = HistoryRecord::from_report(&report);
    assert_eq!(record.server, "Test Server");
    assert!((record.ping_ms - 12.3).abs() < 1e-9);
    assert!((record.download_mbps - 88.1).abs() < 1e-9);
    assert!((record.upload_mbps - 44.9).abs() < 1e-9);
    assert!(record.timestamp > 0);
}

#[test]
fn record_serializes_to_csv_row() {
    let report = common::ReportBuilder::new()
        .server("Test Server")
        .download(88.1)
        .build();
    let record = HistoryRecord::from_report(&report);
    let row = record.to_csv_row();
    assert_eq!(
        row.split(',').count(),
        HistoryRecord::CSV_HEADER.split(',').count(),
        "CSV column count must match header"
    );
    assert!(row.contains("Test Server"));
    assert!(row.contains("88.1"));
}

#[test]
fn csv_quotes_fields_with_commas() {
    let mut report = common::ReportBuilder::new().build();
    report.server_name = "Mumbai, India (ISP \"X\")".to_owned();
    let record = HistoryRecord::from_report(&report);
    let row = record.to_csv_row();
    assert!(row.contains("\"Mumbai, India (ISP \"\"X\"\")\""));
}

#[test]
fn record_serializes_to_json_line() -> Result<(), Box<dyn std::error::Error>> {
    let report = common::ReportBuilder::new().build();
    let record = HistoryRecord::from_report(&report);
    let json = record.to_json()?;
    assert!(!json.contains('\n'), "JSONL must be a single line");
    let parsed: HistoryRecord = serde_json::from_str(&json)?;
    assert_eq!(parsed, record);
    Ok(())
}

#[test]
fn append_creates_directories_and_accumulates_lines() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir().join(format!("netspd-test-{}", std::process::id()));
    let path = dir.join("nested").join("history.jsonl");
    let report = common::ReportBuilder::new().build();
    let record = HistoryRecord::from_report(&report);

    append(&path, &record)?;
    append(&path, &record)?;

    let contents = std::fs::read_to_string(&path)?;
    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(lines.len(), 2);

    std::fs::remove_dir_all(&dir)?;
    Ok(())
}

#[test]
fn load_skips_bad_lines_and_honors_limit() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir().join(format!("netspd-load-{}", std::process::id()));
    let path = dir.join("history.jsonl");
    let report = common::ReportBuilder::new().build();
    let record = HistoryRecord::from_report(&report);

    netspd::app::history::append(&path, &record)?;
    {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new().append(true).open(&path)?;
        writeln!(file, "not json at all")?;
    }
    netspd::app::history::append(&path, &record)?;
    netspd::app::history::append(&path, &record)?;

    let all = netspd::app::history::load(&path, 100);
    assert_eq!(all.len(), 3, "three valid records");
    let limited = netspd::app::history::load(&path, 2);
    assert_eq!(limited.len(), 2, "limit honored");
    assert!(
        netspd::app::history::load(&dir.join("nope.jsonl"), 10).is_empty(),
        "missing file returns empty"
    );

    std::fs::remove_dir_all(&dir)?;
    Ok(())
}
