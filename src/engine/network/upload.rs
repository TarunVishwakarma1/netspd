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
    let payload = build_payload(config.upload_chunk_bytes.max(64 * 1024));

    for _ in 0..config.connections.max(1) {
        workers.spawn(worker(
            client.clone(),
            url.to_owned(),
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

/// A single upload connection: POST the payload with a known length,
/// count it on success, repeat until cancelled.
async fn worker(
    client: Client,
    url: String,
    payload: Bytes,
    counter: Arc<AtomicU64>,
    cancel: CancellationToken,
) {
    while !cancel.is_cancelled() {
        // `Bytes` clones are reference-counted: no copy, and reqwest
        // knows the exact size, so the request carries Content-Length.
        let request = client
            .post(&url)
            .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
            .body(payload.clone())
            .send();

        let response = tokio::select! {
            () = cancel.cancelled() => return,
            result = request => result,
        };

        match response {
            Ok(response) if response.status().is_success() => {
                counter.fetch_add(payload.len() as u64, Ordering::Relaxed);
            }
            // Failed request or error status: back off briefly and retry.
            Ok(_) | Err(_) => {
                tokio::select! {
                    () = cancel.cancelled() => return,
                    () = tokio::time::sleep(RETRY_DELAY) => {}
                }
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
