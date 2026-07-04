//! The engine façade: configuration, server discovery and test execution.

use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::errors::{EngineError, EngineResult};

use super::event::{emit, EngineEvent};
use super::models::{Server, TestReport};
use super::providers::Provider;
use super::scheduler;

/// Address family restriction for measurements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpFamily {
    /// IPv4 only.
    V4,
    /// IPv6 only.
    V6,
}

/// Settings for the ping phase.
#[derive(Debug, Clone, Copy)]
pub struct PingConfig {
    /// Number of latency samples to collect.
    pub samples: u32,
    /// Delay between samples.
    pub interval: Duration,
    /// Per-sample timeout; slower samples count as lost.
    pub timeout: Duration,
}

impl Default for PingConfig {
    fn default() -> Self {
        Self {
            samples: 10,
            interval: Duration::from_millis(100),
            timeout: Duration::from_secs(3),
        }
    }
}

/// Settings shared by the download and upload phases.
#[derive(Debug, Clone, Copy)]
pub struct TransferConfig {
    /// How long each transfer phase runs.
    pub duration: Duration,
    /// Number of parallel connections.
    pub connections: usize,
    /// How often progress snapshots are emitted.
    pub sample_interval: Duration,
    /// EMA smoothing factor for the instantaneous speed.
    pub ema_alpha: f64,
    /// Size of each upload request body, in bytes.
    pub upload_chunk_bytes: usize,
    /// Connection timeout for the shared HTTP client.
    pub connect_timeout: Duration,
    /// Pause between announcing a transfer phase and moving data.
    ///
    /// Front ends use this window for phase transitions (the TUI plays
    /// its ignition sweep in it); the pause is not part of the
    /// measurement.
    pub lead_in: Duration,
}

impl Default for TransferConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_secs(10),
            connections: 4,
            sample_interval: Duration::from_millis(100),
            ema_alpha: 0.2,
            // Large enough that request-per-body round trips don't cap
            // throughput on high-latency links.
            upload_chunk_bytes: 2 * 1024 * 1024,
            connect_timeout: Duration::from_secs(15),
            lead_in: Duration::from_millis(1200),
        }
    }
}

/// Complete engine configuration.
#[derive(Debug, Clone, Copy, Default)]
pub struct EngineConfig {
    /// Ping phase settings.
    pub ping: PingConfig,
    /// Transfer phase settings.
    pub transfer: TransferConfig,
    /// Restrict measurements to one address family.
    pub ip_family: Option<IpFamily>,
}

/// The speed test engine.
///
/// Owns the shared HTTP client and the active [`Provider`]. It has no
/// knowledge of any UI: consumers drive it with [`Engine::run_test`] and
/// observe it through the [`EngineEvent`] stream, which makes it equally
/// usable from a TUI, CLI, GUI or REST API.
pub struct Engine {
    provider: Arc<dyn Provider>,
    client: Client,
    config: EngineConfig,
}

impl Engine {
    /// Creates an engine for the given provider and configuration.
    pub fn new(provider: Arc<dyn Provider>, config: EngineConfig) -> EngineResult<Self> {
        let client = super::network::client::build_client_with_family(
            config.transfer.connect_timeout,
            config.transfer.connections,
            config.ip_family,
        )?;
        Ok(Self {
            provider,
            client,
            config,
        })
    }

    /// Display name of the active provider.
    #[must_use]
    pub fn provider_name(&self) -> &'static str {
        self.provider.name()
    }

    /// Discovers available servers through the active provider.
    ///
    /// Every discovered server is health-probed concurrently; unreachable
    /// entries (dead hosts, invalid TLS certificates) are dropped and the
    /// rest are ordered nearest-first, so the default selection works out
    /// of the box.
    pub async fn load_servers(&self) -> EngineResult<Vec<Server>> {
        let servers = self.provider.fetch_servers().await?;
        if servers.is_empty() {
            return Err(EngineError::NoServers);
        }
        let reachable = super::network::health::filter_reachable(&self.client, servers).await;
        if reachable.is_empty() {
            return Err(EngineError::NoServers);
        }
        Ok(reachable)
    }

    /// Fetches the client's public IP and ISP through `server`.
    ///
    /// Best-effort: failures return `None` rather than an error, because
    /// this information is purely cosmetic.
    pub async fn client_info(&self, server: &Server) -> Option<String> {
        super::network::info::fetch_client_info(&self.client, &server.endpoints.ping).await
    }

    /// Runs a complete test against `server`, streaming progress on
    /// `events`.
    ///
    /// Emits [`EngineEvent::Finished`] on success and
    /// [`EngineEvent::Failed`] on genuine failures; cancellation emits
    /// nothing and returns [`EngineError::Cancelled`].
    pub async fn run_test(
        &self,
        server: &Server,
        events: mpsc::Sender<EngineEvent>,
        cancel: CancellationToken,
    ) -> EngineResult<TestReport> {
        let result =
            scheduler::run_sequence(&self.client, server, &self.config, &events, &cancel).await;
        match &result {
            Ok(report) => {
                emit(
                    &events,
                    EngineEvent::Finished {
                        report: report.clone(),
                    },
                )
                .await?;
            }
            Err(err) if !err.is_cancelled() => {
                emit(
                    &events,
                    EngineEvent::Failed {
                        message: err.to_string(),
                    },
                )
                .await?;
            }
            Err(_) => {}
        }
        result
    }
}
