//! Shared machinery for duration-bounded transfer phases.
//!
//! Download and upload both follow the same shape: a set of worker tasks
//! move bytes and bump a shared counter, while a monitor loop samples the
//! counter, smooths it into speed readings and emits progress events.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use crate::engine::event::{emit, EngineEvent};
use crate::engine::metrics::ThroughputMeter;
use crate::engine::models::{TestPhase, TransferProgress, TransferStats};
use crate::engine::TransferConfig;
use crate::errors::{EngineError, EngineResult};

/// Drives a transfer phase to completion.
///
/// Samples `counter` every `config.sample_interval`, emits
/// [`EngineEvent::Progress`] snapshots and stops the workers once
/// `config.duration` elapses. Cancelling `cancel` aborts immediately with
/// [`EngineError::Cancelled`].
pub(crate) async fn monitor(
    phase: TestPhase,
    config: &TransferConfig,
    counter: Arc<AtomicU64>,
    workers: &mut JoinSet<()>,
    workers_cancel: &CancellationToken,
    events: &mpsc::Sender<EngineEvent>,
    cancel: &CancellationToken,
) -> EngineResult<TransferStats> {
    let started = Instant::now();
    let mut meter = ThroughputMeter::new(started, config.ema_alpha);
    let mut ticker = tokio::time::interval(config.sample_interval);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    // The first tick of a tokio interval fires immediately; consume it so
    // the first sample carries a real time delta.
    ticker.tick().await;

    let outcome = loop {
        tokio::select! {
            () = cancel.cancelled() => break Err(EngineError::Cancelled),
            _ = ticker.tick() => {
                let now = Instant::now();
                let elapsed = now.duration_since(started);
                let total = counter.load(Ordering::Relaxed);
                let snapshot = meter.update(total, now);
                let ratio = (elapsed.as_secs_f64() / config.duration.as_secs_f64()).min(1.0);
                let progress = TransferProgress {
                    phase,
                    bytes_transferred: total,
                    elapsed,
                    current_bps: snapshot.current_bps,
                    average_bps: snapshot.average_bps,
                    peak_bps: snapshot.peak_bps,
                    eta: config.duration.saturating_sub(elapsed),
                    ratio,
                };
                emit(events, EngineEvent::Progress { progress }).await?;
                if elapsed >= config.duration {
                    break Ok(());
                }
            }
        }
    };

    shutdown_workers(workers, workers_cancel).await;
    outcome?;

    let now = Instant::now();
    let duration = now.duration_since(started);
    let total = counter.load(Ordering::Relaxed);
    if total == 0 {
        return Err(EngineError::InvalidResponse(
            "server transferred no data".to_owned(),
        ));
    }
    let snapshot = meter.update(total, now);
    Ok(TransferStats {
        bytes: total,
        duration,
        average_bps: snapshot.average_bps,
        peak_bps: snapshot.peak_bps,
    })
}

/// Signals workers to stop, then aborts and drains any that keep running
/// (e.g. blocked on a stalled read).
async fn shutdown_workers(workers: &mut JoinSet<()>, workers_cancel: &CancellationToken) {
    workers_cancel.cancel();
    workers.abort_all();
    while workers.join_next().await.is_some() {}
}
