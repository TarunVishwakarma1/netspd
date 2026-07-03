//! Streaming download measurement.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use crate::engine::event::EngineEvent;
use crate::engine::models::{TestPhase, TransferStats};
use crate::engine::TransferConfig;
use crate::errors::EngineResult;

use super::transfer;

/// Pause between retries after a failed request, so a broken server does
/// not turn the worker into a busy loop.
const RETRY_DELAY: Duration = Duration::from_millis(250);

/// Runs the download phase.
///
/// Spawns `config.connections` workers that stream the payload without ever
/// buffering it, counting bytes as chunks arrive. Progress is emitted on
/// `events`; the phase ends after `config.duration` or on cancellation.
pub async fn run_download(
    client: &Client,
    url: &str,
    config: &TransferConfig,
    events: &mpsc::Sender<EngineEvent>,
    cancel: &CancellationToken,
) -> EngineResult<TransferStats> {
    let counter = Arc::new(AtomicU64::new(0));
    let workers_cancel = cancel.child_token();
    let mut workers = JoinSet::new();

    for _ in 0..config.connections.max(1) {
        workers.spawn(worker(
            client.clone(),
            url.to_owned(),
            Arc::clone(&counter),
            workers_cancel.clone(),
        ));
    }

    transfer::monitor(
        TestPhase::Download,
        config,
        counter,
        &mut workers,
        &workers_cancel,
        events,
        cancel,
    )
    .await
}

/// A single download connection: request, stream, count, repeat.
///
/// Individual request failures are retried after a short delay; the worker
/// only stops when cancelled.
async fn worker(client: Client, url: String, counter: Arc<AtomicU64>, cancel: CancellationToken) {
    while !cancel.is_cancelled() {
        let response = tokio::select! {
            () = cancel.cancelled() => return,
            result = client.get(&url).send() => result,
        };

        let mut response = match response {
            Ok(response) if response.status().is_success() => response,
            // Failed request or error status: back off briefly and retry.
            Ok(_) | Err(_) => {
                tokio::select! {
                    () = cancel.cancelled() => return,
                    () = tokio::time::sleep(RETRY_DELAY) => continue,
                }
            }
        };

        loop {
            let chunk = tokio::select! {
                () = cancel.cancelled() => return,
                result = response.chunk() => result,
            };
            match chunk {
                Ok(Some(bytes)) => {
                    counter.fetch_add(bytes.len() as u64, Ordering::Relaxed);
                }
                // Stream finished cleanly or errored mid-body; either way,
                // start a fresh request.
                Ok(None) | Err(_) => break,
            }
        }
    }
}
