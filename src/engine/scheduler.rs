//! Orders the phases of a speed test run.

use reqwest::Client;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::errors::EngineResult;

use super::engine::EngineConfig;
use super::engine::IpFamily;
use super::event::{emit, EngineEvent};
use super::models::{Bufferbloat, IpVersion, LatencyStats, Server, TestPhase, TestReport};
use super::network::{download, icmp, loaded_latency, ping, upload};

/// Runs ping, download and upload in sequence against one server and
/// assembles the final report.
///
/// Each phase announces itself with [`EngineEvent::PhaseStarted`] and
/// publishes its results before the next phase begins. Cancellation is
/// honored at every await point.
pub(crate) async fn run_sequence(
    client: &Client,
    server: &Server,
    config: &EngineConfig,
    events: &mpsc::Sender<EngineEvent>,
    cancel: &CancellationToken,
) -> EngineResult<TestReport> {
    emit(
        events,
        EngineEvent::PhaseStarted {
            phase: TestPhase::Ping,
        },
    )
    .await?;
    // HTTP latency and ICMP loss probe run side by side; ICMP is
    // best-effort and refines the loss figure when sockets allow it.
    let (latency, icmp_loss) = tokio::join!(
        ping::measure_latency(client, &server.endpoints.ping, &config.ping, events, cancel),
        measure_icmp_loss(server, cancel),
    );
    let mut latency: LatencyStats = latency?;
    if let Some(loss) = icmp_loss {
        latency.packet_loss_pct = loss;
    }
    emit(events, EngineEvent::PingFinished { stats: latency }).await?;

    emit(
        events,
        EngineEvent::PhaseStarted {
            phase: TestPhase::Download,
        },
    )
    .await?;
    lead_in(config, cancel).await?;
    // Latency is sampled while the link is saturated: the difference
    // against idle latency is the bufferbloat measurement.
    let sampler_stop = cancel.child_token();
    let (download_stats, loaded_down_ms) = tokio::join!(
        async {
            let result = download::run_download(
                client,
                &server.endpoints.download,
                &config.transfer,
                events,
                cancel,
            )
            .await;
            sampler_stop.cancel();
            result
        },
        loaded_latency::sample_until_stopped(client, &server.endpoints.ping, &sampler_stop),
    );
    let download_stats = download_stats?;
    emit(
        events,
        EngineEvent::TransferFinished {
            phase: TestPhase::Download,
            stats: download_stats,
        },
    )
    .await?;

    emit(
        events,
        EngineEvent::PhaseStarted {
            phase: TestPhase::Upload,
        },
    )
    .await?;
    lead_in(config, cancel).await?;
    let sampler_stop = cancel.child_token();
    let (upload_stats, loaded_up_ms) = tokio::join!(
        async {
            let result = upload::run_upload(
                client,
                &server.endpoints.upload,
                &config.transfer,
                events,
                cancel,
            )
            .await;
            sampler_stop.cancel();
            result
        },
        loaded_latency::sample_until_stopped(client, &server.endpoints.ping, &sampler_stop),
    );
    let upload_stats = upload_stats?;
    emit(
        events,
        EngineEvent::TransferFinished {
            phase: TestPhase::Upload,
            stats: upload_stats,
        },
    )
    .await?;

    let bufferbloat = match (loaded_down_ms, loaded_up_ms) {
        (Some(down), Some(up)) => Some(Bufferbloat::new(latency.average_ms, down, up)),
        _ => None,
    };

    let ip_version = config.ip_family.map(|f| match f {
        IpFamily::V4 => IpVersion::V4,
        IpFamily::V6 => IpVersion::V6,
    });
    Ok(TestReport {
        server_name: server.name.clone(),
        latency,
        download: download_stats,
        upload: upload_stats,
        bufferbloat,
        ip_version,
    })
}

/// Runs the ICMP loss probe against the server's host, when derivable.
async fn measure_icmp_loss(server: &Server, cancel: &CancellationToken) -> Option<f64> {
    let host = icmp::host_of_url(&server.endpoints.ping)?;
    icmp::measure_loss(&host, cancel).await
}

/// Waits out the configured lead-in between a phase announcement and its
/// first byte, so front ends can play phase transitions against a still
/// needle. Cancellation is honored during the wait.
async fn lead_in(config: &EngineConfig, cancel: &CancellationToken) -> EngineResult<()> {
    if config.transfer.lead_in.is_zero() {
        return Ok(());
    }
    tokio::select! {
        () = cancel.cancelled() => Err(crate::errors::EngineError::Cancelled),
        () = tokio::time::sleep(config.transfer.lead_in) => Ok(()),
    }
}
