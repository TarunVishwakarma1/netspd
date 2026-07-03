//! Shared HTTP client construction.

use std::time::Duration;

use reqwest::Client;

use crate::errors::EngineResult;

/// User agent sent with every request.
const USER_AGENT: &str = concat!("netspd/", env!("CARGO_PKG_VERSION"));

/// Builds the single, shared HTTP client used by every phase.
///
/// The client uses rustls for TLS, keeps connections pooled per host so
/// parallel transfer workers can reuse sockets, and applies a connect
/// timeout only — transfer phases are bounded by their own deadlines, so a
/// total request timeout would cut healthy long-running streams.
pub fn build_client(connect_timeout: Duration, pool_size: usize) -> EngineResult<Client> {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .use_rustls_tls()
        .connect_timeout(connect_timeout)
        .pool_max_idle_per_host(pool_size.max(1))
        .tcp_nodelay(true)
        .build()?;
    Ok(client)
}
