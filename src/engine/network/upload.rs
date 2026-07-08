//! Streaming upload measurement.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use reqwest::Client;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use crate::engine::event::EngineEvent;
use crate::engine::models::{TestPhase, TransferStats};
use crate::engine::TransferConfig;
use crate::errors::EngineResult;

use super::transfer;

/// Pause between retries after a failed request.
const RETRY_DELAY: Duration = Duration::from_millis(250);

/// Size of the first, smaller request each worker sends.
///
/// The steady-state body is deliberately large (multiple megabytes) so a
/// high-latency link isn't capped by per-request round trips. But on a slow
/// uplink one such body can take several seconds to complete, during which
/// the counter — and therefore the gauge — sits at zero. A small warm-up
/// request completes quickly, so the meter reads a real speed within the
/// first sampling intervals instead of snapping up once the first full body
/// finishes. Bytes are counted on completion just like any other request.
const WARMUP_BYTES: usize = 128 * 1024;

/// Runs the upload phase.
///
/// One incompressible payload is generated once and shared by every
/// worker through cheap reference-counted [`Bytes`] clones. Bodies are
/// sent with an explicit `Content-Length` — never chunked — because
/// widely-deployed speed test backends (Ookla's `upload.php` among
/// them) hang forever on chunked requests. Bytes are counted as each
/// request completes; with multi-megabyte bodies across several
/// connections that still lands several updates per sampling interval.
pub async fn run_upload(
    client: &Client,
    url: &str,
    config: &TransferConfig,
    events: &mpsc::Sender<EngineEvent>,
    cancel: &CancellationToken,
) -> EngineResult<TransferStats> {
    let counter = Arc::new(AtomicU64::new(0));
    let workers_cancel = cancel.child_token();
    let mut workers = JoinSet::new();
    let full = config.upload_chunk_bytes.max(64 * 1024);
    let payload = build_payload(full);
    // The warm-up body is a prefix of the full payload, so it costs no extra
    // allocation — `Bytes::slice` is a reference-counted view.
    let warmup = payload.slice(0..WARMUP_BYTES.min(full));

    for _ in 0..config.connections.max(1) {
        workers.spawn(worker(
            client.clone(),
            url.to_owned(),
            warmup.clone(),
            payload.clone(),
            Arc::clone(&counter),
            workers_cancel.clone(),
        ));
    }

    transfer::monitor(
        TestPhase::Upload,
        config,
        counter,
        &mut workers,
        &workers_cancel,
        events,
        cancel,
    )
    .await
}

/// A single upload connection: send a quick warm-up body for early
/// feedback, then POST the full payload in a loop until cancelled. Every
/// request carries a known length, and bytes are counted on success.
async fn worker(
    client: Client,
    url: String,
    warmup: Bytes,
    payload: Bytes,
    counter: Arc<AtomicU64>,
    cancel: CancellationToken,
) {
    if !cancel.is_cancelled() {
        send_once(&client, &url, &warmup, &counter, &cancel).await;
    }
    while !cancel.is_cancelled() {
        send_once(&client, &url, &payload, &counter, &cancel).await;
    }
}

/// Sends one body and, on success, adds its length to the counter. On
/// failure (or cancellation mid-flight) it backs off briefly. Returns when
/// the attempt is resolved so the caller can loop or exit.
async fn send_once(
    client: &Client,
    url: &str,
    body: &Bytes,
    counter: &Arc<AtomicU64>,
    cancel: &CancellationToken,
) {
    // `Bytes` clones are reference-counted: no copy, and reqwest knows the
    // exact size, so the request carries Content-Length (never chunked —
    // some backends hang forever on chunked upload bodies).
    let request = client
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
        .body(body.clone())
        .send();

    let response = tokio::select! {
        () = cancel.cancelled() => return,
        result = request => result,
    };

    match response {
        Ok(response) if response.status().is_success() => {
            counter.fetch_add(body.len() as u64, Ordering::Relaxed);
        }
        // Failed request or error status: back off briefly and retry.
        Ok(_) | Err(_) => {
            tokio::select! {
                () = cancel.cancelled() => {}
                () = tokio::time::sleep(RETRY_DELAY) => {}
            }
        }
    }
}

/// Builds the shared upload payload.
///
/// The payload is pseudo-random (xorshift) so transparent compression
/// anywhere on the path cannot inflate the measured speed.
fn build_payload(total_bytes: usize) -> Bytes {
    let mut data = vec![0_u8; total_bytes];
    let mut state: u64 = 0x9e37_79b9_7f4a_7c15;
    for slot in &mut data {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        *slot = (state & 0xff) as u8;
    }
    Bytes::from(data)
}
