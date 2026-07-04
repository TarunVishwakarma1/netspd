//! The provider abstraction: where speed test servers come from.
//!
//! The engine only ever talks to the [`Provider`] trait. Adding Ookla,
//! Fast.com or a self-hosted backend means implementing this one trait —
//! nothing else in the codebase changes.

mod custom;
mod fast;
mod librespeed;
mod ookla;

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::errors::EngineResult;

use super::models::Server;

pub use custom::CustomProvider;
pub use fast::{parse_targets, FastProvider};
pub use librespeed::{builtin_servers, parse_server_list, LibreSpeedProvider};
pub use ookla::{parse_server_list as parse_ookla_server_list, OoklaProvider};

/// A source of speed test servers.
///
/// Implementations resolve every phase URL up front, so the engine and the
/// networking layer stay completely provider-agnostic.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Short display name of this provider.
    fn name(&self) -> &'static str;

    /// Discovers available servers, already validated and normalized.
    async fn fetch_servers(&self) -> EngineResult<Vec<Server>>;
}

/// The providers a user can select in configuration or with
/// `--provider`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    /// The open-source LibreSpeed backend network.
    #[default]
    Librespeed,
    /// Ookla's speedtest.net server network.
    Ookla,
    /// Netflix's Fast.com CDN targets.
    Fast,
    /// User-defined servers from `[[servers]]` in the configuration.
    Custom,
}

impl ProviderKind {
    /// Human-readable label for this provider kind.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Librespeed => "LibreSpeed",
            Self::Ookla => "Ookla",
            Self::Fast => "Fast.com",
            Self::Custom => "Custom",
        }
    }

    /// One-line description shown by `--list-providers`.
    #[must_use]
    pub fn description(self) -> &'static str {
        match self {
            Self::Librespeed => "open-source community server network",
            Self::Ookla => "Ookla speedtest.net",
            Self::Fast => "Netflix Fast.com",
            Self::Custom => "your own [[servers]] in config.toml (LAN, pods, self-hosted backends)",
        }
    }

    /// All selectable provider kinds, in display order.
    pub const ALL: [Self; 4] = [Self::Librespeed, Self::Ookla, Self::Fast, Self::Custom];
}

impl std::str::FromStr for ProviderKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "librespeed" => Ok(Self::Librespeed),
            "ookla" | "speedtest" => Ok(Self::Ookla),
            "fast" | "fast.com" => Ok(Self::Fast),
            "custom" => Ok(Self::Custom),
            other => Err(format!(
                "unknown provider {other:?}; expected librespeed, ookla, fast or custom"
            )),
        }
    }
}

/// Instantiates the configured provider.
///
/// The `custom` provider serves exactly the `[[servers]]` entries from
/// configuration. For backward compatibility those entries also override
/// LibreSpeed's discovery when non-empty; Ookla and Fast.com manage
/// their own server pools and ignore them.
pub fn create(kind: ProviderKind, custom_servers: Vec<Server>) -> EngineResult<Arc<dyn Provider>> {
    match kind {
        ProviderKind::Librespeed => Ok(Arc::new(LibreSpeedProvider::new(custom_servers)?)),
        ProviderKind::Ookla => Ok(Arc::new(OoklaProvider::new()?)),
        ProviderKind::Fast => Ok(Arc::new(FastProvider::new()?)),
        ProviderKind::Custom => Ok(Arc::new(CustomProvider::new(custom_servers)?)),
    }
}
