//! Configuration file discovery and parsing.

use std::path::{Path, PathBuf};

use crate::errors::{ConfigError, ConfigResult};

use super::Settings;

/// File name of the configuration inside each search location.
const FILE_NAME: &str = "config.toml";

/// Loads settings from the first configuration file found, falling back to
/// defaults when none exists.
///
/// Search order:
/// 1. `$XDG_CONFIG_HOME/netspd/config.toml` (or the platform equivalent)
/// 2. `./config/config.toml`
pub fn load() -> ConfigResult<Settings> {
    for path in candidate_paths() {
        if path.is_file() {
            return load_from(&path);
        }
    }
    Ok(Settings::default())
}

/// Loads settings from an explicit path.
pub fn load_from(path: &Path) -> ConfigResult<Settings> {
    let raw = std::fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.to_owned(),
        source,
    })?;
    toml::from_str(&raw).map_err(|source| ConfigError::Parse {
        path: path.to_owned(),
        source: Box::new(source),
    })
}

/// The locations searched for a configuration file, in priority order.
fn candidate_paths() -> Vec<PathBuf> {
    let mut paths = Vec::with_capacity(2);
    if let Some(config_dir) = dirs::config_dir() {
        paths.push(config_dir.join("netspd").join(FILE_NAME));
    }
    paths.push(PathBuf::from("config").join(FILE_NAME));
    paths
}
