//! User-defined provider: servers declared in configuration.
//!
//! For self-hosted backends, LAN speed testing, or any HTTP endpoint set
//! the user controls. Requirements on the endpoints are minimal:
//!
//! - **ping**: answers GET with any 2xx quickly;
//! - **download**: answers GET with a large (or unbounded) body;
//! - **upload**: accepts POST bodies with a 2xx.
//!
//! A stock LibreSpeed container satisfies all three, but so does any
//! plain web server with a big file and an upload sink.

use async_trait::async_trait;

use crate::engine::models::Server;
use crate::errors::{EngineError, EngineResult};

use super::Provider;

/// A provider serving exactly the servers the user configured.
pub struct CustomProvider {
    servers: Vec<Server>,
}

impl CustomProvider {
    /// Creates the provider from `[[servers]]` entries.
    ///
    /// Fails when the list is empty: selecting `provider = "custom"`
    /// without declaring servers is a configuration error worth
    /// surfacing immediately rather than at test time.
    pub fn new(servers: Vec<Server>) -> EngineResult<Self> {
        if servers.is_empty() {
            return Err(EngineError::InvalidResponse(
                "provider \"custom\" requires at least one [[servers]] entry in config.toml"
                    .to_owned(),
            ));
        }
        Ok(Self { servers })
    }
}

#[async_trait]
impl Provider for CustomProvider {
    fn name(&self) -> &'static str {
        "Custom"
    }

    async fn fetch_servers(&self) -> EngineResult<Vec<Server>> {
        Ok(self.servers.clone())
    }
}
