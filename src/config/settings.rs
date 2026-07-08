//! The configuration schema and its mapping onto engine types.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::engine::models::Server;
use crate::engine::providers::ProviderKind;
use crate::engine::{EngineConfig, PingConfig, TransferConfig};

/// Top-level application settings, loaded from `config.toml`.
///
/// Every field has a default, so an empty or missing file yields a fully
/// working configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    /// Name of the active theme (matched case-insensitively).
    pub theme: String,
    /// UI refresh rate in frames per second, clamped to `1..=60`.
    pub refresh_rate: u16,
    /// The speed test provider to use.
    pub provider: ProviderKind,
    /// Multiplier applied to UI animation speed.
    pub animation_speed: f64,
    /// Engine tuning parameters.
    pub engine: EngineSection,
    /// Repeat the test on this interval, e.g. `"15m"` (min 30s).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_interval: Option<String>,
    /// Send a desktop notification when each test finishes (macOS / Linux).
    pub notify: bool,
    /// Custom servers overriding provider discovery.
    pub servers: Vec<ServerEntry>,
    /// Background wallpaper rendered behind all UI elements.
    pub wallpaper: WallpaperSection,
}

/// What to render behind the UI — configured under `[wallpaper]`.
///
/// ```toml
/// [wallpaper]
/// kind = "gradient"
/// from = "#0d1117"   # top colour (optional, defaults to theme background)
/// to   = "#000000"   # bottom colour (optional, defaults to half-darkened bg)
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct WallpaperSection {
    /// `"none"` (default solid theme background) or `"gradient"`.
    pub kind: WallpaperKind,
    /// Gradient start (top) colour as `#rrggbb`. Defaults to the active
    /// theme's background colour when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// Gradient end (bottom) colour as `#rrggbb`. Defaults to a 50%-darkened
    /// version of the theme background when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

/// The type of background wallpaper.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WallpaperKind {
    /// Solid fill using the theme's `background` colour (default).
    #[default]
    None,
    /// Vertical colour gradient from `from` (top) to `to` (bottom).
    Gradient,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "default".to_owned(),
            refresh_rate: 30,
            provider: ProviderKind::default(),
            animation_speed: 1.0,
            engine: EngineSection::default(),
            repeat_interval: None,
            notify: true,
            servers: Vec::new(),
            wallpaper: WallpaperSection::default(),
        }
    }
}

/// Engine tuning parameters from the `[engine]` table.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EngineSection {
    /// Number of latency samples collected during the ping phase.
    pub ping_samples: u32,
    /// Delay between ping samples, in milliseconds.
    pub ping_interval_ms: u64,
    /// Duration of each transfer phase, in seconds.
    pub duration_secs: u64,
    /// Number of parallel connections per transfer phase.
    pub connections: usize,
    /// Per-connection timeout, in seconds.
    pub timeout_secs: u64,
    /// Size of each upload body, in kilobytes.
    pub upload_chunk_kb: usize,
}

impl Default for EngineSection {
    fn default() -> Self {
        let ping = PingConfig::default();
        let transfer = TransferConfig::default();
        Self {
            ping_samples: ping.samples,
            ping_interval_ms: ping.interval.as_millis() as u64,
            duration_secs: transfer.duration.as_secs(),
            connections: transfer.connections,
            timeout_secs: transfer.connect_timeout.as_secs(),
            upload_chunk_kb: transfer.upload_chunk_bytes / 1024,
        }
    }
}

/// One custom server from a `[[servers]]` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerEntry {
    /// Display name.
    pub name: String,
    /// Base URL of the backend, e.g. `https://host/backend/`.
    pub url: String,
    /// Download path relative to the base URL.
    #[serde(default = "default_download_path")]
    pub download_path: String,
    /// Upload path relative to the base URL.
    #[serde(default = "default_upload_path")]
    pub upload_path: String,
    /// Ping path relative to the base URL.
    #[serde(default = "default_ping_path")]
    pub ping_path: String,
}

fn default_download_path() -> String {
    "garbage.php?ckSize=100".to_owned()
}

fn default_upload_path() -> String {
    "empty.php".to_owned()
}

fn default_ping_path() -> String {
    "empty.php".to_owned()
}

impl Settings {
    /// The UI tick period derived from the configured refresh rate.
    #[must_use]
    pub fn tick_rate(&self) -> Duration {
        let fps = u64::from(self.refresh_rate.clamp(1, 60));
        Duration::from_millis(1000 / fps)
    }

    /// The animation speed multiplier, clamped to a sane range.
    #[must_use]
    pub fn animation_speed(&self) -> f64 {
        if self.animation_speed.is_finite() {
            self.animation_speed.clamp(0.1, 5.0)
        } else {
            1.0
        }
    }

    /// Maps the user-facing settings onto the engine's configuration.
    #[must_use]
    pub fn engine_config(&self) -> EngineConfig {
        let defaults = EngineConfig::default();
        EngineConfig {
            ping: PingConfig {
                samples: self.engine.ping_samples.clamp(3, 100),
                interval: Duration::from_millis(self.engine.ping_interval_ms.clamp(10, 2000)),
                timeout: defaults.ping.timeout,
            },
            transfer: TransferConfig {
                duration: Duration::from_secs(self.engine.duration_secs.clamp(3, 60)),
                connections: self.engine.connections.clamp(1, 16),
                sample_interval: defaults.transfer.sample_interval,
                ema_alpha: defaults.transfer.ema_alpha,
                upload_chunk_bytes: self.engine.upload_chunk_kb.clamp(64, 8192) * 1024,
                connect_timeout: Duration::from_secs(self.engine.timeout_secs.clamp(2, 120)),
                lead_in: defaults.transfer.lead_in,
            },
            ip_family: None,
        }
    }

    /// The parsed repeat interval, if configured and valid.
    ///
    /// Invalid strings are treated as unset rather than fatal: a typo in
    /// the config should not block a one-off test.
    #[must_use]
    pub fn repeat_interval(&self) -> Option<Duration> {
        self.repeat_interval
            .as_deref()
            .and_then(|value| crate::utils::duration::parse_interval(value).ok())
    }

    /// Applies command line overrides on top of the file configuration.
    pub fn apply_overrides(&mut self, duration_secs: Option<u64>, connections: Option<usize>) {
        if let Some(duration) = duration_secs {
            self.engine.duration_secs = duration;
        }
        if let Some(connections) = connections {
            self.engine.connections = connections;
        }
    }

    /// Resolves the custom `[[servers]]` entries into engine servers.
    #[must_use]
    pub fn custom_servers(&self) -> Vec<Server> {
        self.servers
            .iter()
            .filter(|entry| !entry.name.trim().is_empty() && !entry.url.trim().is_empty())
            .map(|entry| {
                Server::from_base(
                    entry.name.trim(),
                    &entry.url,
                    &entry.download_path,
                    &entry.upload_path,
                    &entry.ping_path,
                )
            })
            .collect()
    }
}
