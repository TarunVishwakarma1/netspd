//! Shared HTTP client construction.

use std::net::IpAddr;
use std::time::Duration;

use reqwest::Client;

use crate::engine::IpFamily;
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
    build_client_with_family(connect_timeout, pool_size, None)
}

/// [`build_client`] with an optional address-family restriction:
/// binding the local side to `0.0.0.0` (or `::`) forces every connection
/// onto that family.
pub fn build_client_with_family(
    connect_timeout: Duration,
    pool_size: usize,
    family: Option<IpFamily>,
) -> EngineResult<Client> {
    let mut builder = Client::builder()
        .user_agent(USER_AGENT)
        .use_rustls_tls()
        .connect_timeout(connect_timeout)
        .pool_max_idle_per_host(pool_size.max(1))
        .tcp_nodelay(true);
    builder = match family {
        Some(IpFamily::V4) => builder.local_address(IpAddr::from([0, 0, 0, 0])),
        Some(IpFamily::V6) => builder.local_address(IpAddr::from([0u16; 8])),
        None => builder,
    };
    Ok(builder.build()?)
}
