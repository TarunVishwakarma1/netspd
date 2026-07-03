//! The provider abstraction: where speed test servers come from.
//!
//! The engine only ever talks to the [`Provider`] trait. Adding Ookla,
//! Fast.com or a self-hosted backend means implementing this one trait —
//! nothing else in the codebase changes.

mod librespeed;

use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;

use crate::errors::EngineResult;

use super::models::Server;

pub use librespeed::{builtin_servers, parse_server_list, LibreSpeedProvider};

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

/// The providers a user can select in configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    /// The open-source LibreSpeed backend network.
    #[default]
    Librespeed,
}

impl ProviderKind {
    /// Human-readable label for this provider kind.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Librespeed => "LibreSpeed",
        }
    }
}

/// Instantiates the configured provider.
///
/// `custom_servers` (from user configuration) take precedence over the
/// provider's own discovery when non-empty.
pub fn create(kind: ProviderKind, custom_servers: Vec<Server>) -> EngineResult<Arc<dyn Provider>> {
    match kind {
        ProviderKind::Librespeed => Ok(Arc::new(LibreSpeedProvider::new(custom_servers)?)),
    }
}
