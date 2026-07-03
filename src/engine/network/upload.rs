//! Streaming upload measurement.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use reqwest::{Body, Client};
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

/// Size of the individual body chunks streamed to the server.
const STREAM_CHUNK: usize = 64 * 1024;

/// Runs the upload phase.
///
/// One incompressible payload is generated once and shared by every worker
/// through cheap reference-counted [`Bytes`] slices — no per-request
/// allocation. Bytes are counted as the request body streams out.
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
    let chunks = payload_chunks(config.upload_chunk_bytes.max(STREAM_CHUNK));

    for _ in 0..config.connections.max(1) {
        workers.spawn(worker(
            client.clone(),
            url.to_owned(),
            chunks.clone(),
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

/// A single upload connection: POST the payload as a counted stream,
/// repeat until cancelled.
async fn worker(
    client: Client,
    url: String,
    chunks: Vec<Bytes>,
    counter: Arc<AtomicU64>,
    cancel: CancellationToken,
) {
    while !cancel.is_cancelled() {
        // Cloning the chunk vector is cheap: each element is a
        // reference-counted slice of the one shared payload buffer.
        let body = counting_body(chunks.clone(), &counter);
        let request = client
            .post(&url)
            .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
            .body(body)
            .send();

        let response = tokio::select! {
            () = cancel.cancelled() => return,
            result = request => result,
        };

        if response.is_err() {
            tokio::select! {
                () = cancel.cancelled() => return,
                () = tokio::time::sleep(RETRY_DELAY) => {}
            }
        }
    }
}

/// Wraps the shared payload in a stream that bumps `counter` as each chunk
/// is pulled by the HTTP client, giving fine-grained progress without
/// copying the payload.
fn counting_body(chunks: Vec<Bytes>, counter: &Arc<AtomicU64>) -> Body {
    let counter = Arc::clone(counter);
    let stream = futures_util::stream::iter(chunks.into_iter().map(move |chunk| {
        counter.fetch_add(chunk.len() as u64, Ordering::Relaxed);
        Ok::<Bytes, std::io::Error>(chunk)
    }));
    Body::wrap_stream(stream)
}

/// Builds the shared upload payload as zero-copy slices of one buffer.
///
/// The payload is pseudo-random (xorshift) so transparent compression
/// anywhere on the path cannot inflate the measured speed.
fn payload_chunks(total_bytes: usize) -> Vec<Bytes> {
    let mut data = vec![0_u8; total_bytes];
    let mut state: u64 = 0x9e37_79b9_7f4a_7c15;
    for slot in &mut data {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        *slot = (state & 0xff) as u8;
    }
    let payload = Bytes::from(data);
    (0..payload.len())
        .step_by(STREAM_CHUNK)
        .map(|start| payload.slice(start..payload.len().min(start + STREAM_CHUNK)))
        .collect()
}
