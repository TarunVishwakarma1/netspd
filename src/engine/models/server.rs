//! Speed test server descriptions.

/// Fully-resolved URLs for each measurement phase on a single server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoints {
    /// URL used for latency samples (small, cache-busted GET requests).
    pub ping: String,
    /// URL streaming an effectively unbounded payload for downloads.
    pub download: String,
    /// URL accepting arbitrary POST bodies for uploads.
    pub upload: String,
}

/// A single speed test server offered by a provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Server {
    /// Display name, usually including the server's location.
    pub name: String,
    /// Short secondary label, typically the server's host name.
    pub description: String,
    /// Resolved endpoints for every test phase.
    pub endpoints: Endpoints,
}

impl Server {
    /// Creates a server from a base URL and relative endpoint paths.
    ///
    /// The base URL is normalized: protocol-relative URLs (`//host/`) get
    /// an `https:` scheme and a trailing slash is ensured so relative
    /// paths join cleanly.
    #[must_use]
    pub fn from_base(
        name: impl Into<String>,
        base_url: &str,
        download_path: &str,
        upload_path: &str,
        ping_path: &str,
    ) -> Self {
        let base = normalize_base(base_url);
        let description = host_of(&base);
        Self {
            name: name.into(),
            description,
            endpoints: Endpoints {
                ping: join(&base, ping_path),
                download: join(&base, download_path),
                upload: join(&base, upload_path),
            },
        }
    }
}

/// Ensures a scheme and a trailing slash on a base URL.
fn normalize_base(base: &str) -> String {
    let trimmed = base.trim();
    let with_scheme = if let Some(rest) = trimmed.strip_prefix("//") {
        format!("https://{rest}")
    } else if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_owned()
    } else {
        format!("https://{trimmed}")
    };
    if with_scheme.ends_with('/') {
        with_scheme
    } else {
        format!("{with_scheme}/")
    }
}

/// Joins a normalized base URL with a relative path.
fn join(base: &str, path: &str) -> String {
    format!("{base}{}", path.trim_start_matches('/'))
}

/// Extracts the host portion of a normalized URL for display purposes.
fn host_of(url: &str) -> String {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    without_scheme
        .split('/')
        .next()
        .unwrap_or(without_scheme)
        .to_owned()
}
