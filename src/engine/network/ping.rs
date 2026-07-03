//! HTTP latency measurement.

use std::time::Instant;

use reqwest::Client;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::engine::event::{emit, EngineEvent};
use crate::engine::metrics::LatencyCalculator;
use crate::engine::models::LatencyStats;
use crate::engine::PingConfig;
use crate::errors::{EngineError, EngineResult};

/// Measures round-trip latency with repeated small HTTP requests.
///
/// Each sample is a cache-busted GET whose time-to-headers approximates
/// round-trip time. Failed samples count toward packet loss instead of
/// aborting the phase; outliers are trimmed during aggregation.
pub async fn measure_latency(
    client: &Client,
    url: &str,
    config: &PingConfig,
    events: &mpsc::Sender<EngineEvent>,
    cancel: &CancellationToken,
) -> EngineResult<LatencyStats> {
    let mut calculator = LatencyCalculator::new();
    let mut last_failure: Option<String> = None;

    for sequence in 1..=config.samples {
        if cancel.is_cancelled() {
            return Err(EngineError::Cancelled);
        }

        let request = client
            .get(cache_busted(url, sequence))
            .header(reqwest::header::CACHE_CONTROL, "no-cache")
            .send();
        let started = Instant::now();

        tokio::select! {
            () = cancel.cancelled() => return Err(EngineError::Cancelled),
            outcome = tokio::time::timeout(config.timeout, request) => {
                match outcome {
                    Ok(Ok(response)) if response.status().is_success() => {
                        let latency_ms = started.elapsed().as_secs_f64() * 1000.0;
                        calculator.record(latency_ms);
                        emit(events, EngineEvent::PingSample { sequence, latency_ms }).await?;
                    }
                    // Non-success status, transport error or timeout all
                    // count as a lost sample; remember why, so a fully
                    // failed phase can report a diagnosable cause.
                    Ok(Ok(response)) => {
                        calculator.record_failure();
                        last_failure = Some(format!("server returned HTTP {}", response.status()));
                    }
                    Ok(Err(err)) => {
                        calculator.record_failure();
                        last_failure = Some(root_cause(&err));
                    }
                    Err(_) => {
                        calculator.record_failure();
                        last_failure = Some(format!(
                            "no response within {:.0}s",
                            config.timeout.as_secs_f64()
                        ));
                    }
                }
            }
        }

        if sequence < config.samples {
            tokio::select! {
                () = cancel.cancelled() => return Err(EngineError::Cancelled),
                () = tokio::time::sleep(config.interval) => {}
            }
        }
    }

    calculator.stats().ok_or_else(|| EngineError::NoSamples {
        reason: last_failure.unwrap_or_else(|| "no requests were attempted".to_owned()),
    })
}

/// Digs to the deepest source of an error chain.
///
/// `reqwest::Error`'s own message is a generic wrapper ("error sending
/// request for url …"); the root cause carries the useful part, e.g.
/// "invalid peer certificate: NotValidForName".
fn root_cause(err: &dyn std::error::Error) -> String {
    let mut current = err;
    while let Some(source) = current.source() {
        current = source;
    }
    current.to_string()
}

/// Appends a cache-busting query parameter so intermediaries cannot serve
/// cached responses.
fn cache_busted(url: &str, sequence: u32) -> String {
    let separator = if url.contains('?') { '&' } else { '?' };
    format!("{url}{separator}r={sequence}")
}
