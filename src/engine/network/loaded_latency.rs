//! Latency sampling while the link is saturated (bufferbloat).
//!
//! A well-behaved connection keeps latency flat while moving data at
//! full speed; a bloated one queues packets and lag spikes. This sampler
//! pings the server's small endpoint throughout a transfer phase and
//! reports the median round-trip, which the scheduler compares against
//! the idle latency.

use std::time::{Duration, Instant};

use reqwest::Client;
use tokio_util::sync::CancellationToken;

use crate::engine::metrics::statistics;

/// Pause between samples; sparse enough not to affect the measurement.
const SAMPLE_INTERVAL: Duration = Duration::from_millis(400);

/// Per-sample timeout; a saturated link can be very slow.
const SAMPLE_TIMEOUT: Duration = Duration::from_secs(4);

/// Minimum samples for a meaningful median.
const MIN_SAMPLES: usize = 3;

/// Samples latency against `url` until `stop` is cancelled, returning
/// the median in milliseconds.
///
/// Best-effort: too few samples (phase too short, requests all failing)
/// yields `None` and bufferbloat simply goes unreported.
pub async fn sample_until_stopped(
    client: &Client,
    url: &str,
    stop: &CancellationToken,
) -> Option<f64> {
    let mut samples = Vec::new();
    let mut sequence: u32 = 0;
    loop {
        sequence = sequence.wrapping_add(1);
        let separator = if url.contains('?') { '&' } else { '?' };
        let request = client
            .get(format!("{url}{separator}bloat={sequence}"))
            .header(reqwest::header::CACHE_CONTROL, "no-cache")
            .send();
        let started = Instant::now();

        tokio::select! {
            () = stop.cancelled() => break,
            outcome = tokio::time::timeout(SAMPLE_TIMEOUT, request) => {
                if let Ok(Ok(response)) = outcome {
                    if response.status().is_success() {
                        samples.push(started.elapsed().as_secs_f64() * 1000.0);
                    }
                }
            }
        }

        tokio::select! {
            () = stop.cancelled() => break,
            () = tokio::time::sleep(SAMPLE_INTERVAL) => {}
        }
    }

    if samples.len() < MIN_SAMPLES {
        return None;
    }
    statistics::median(&samples)
}
