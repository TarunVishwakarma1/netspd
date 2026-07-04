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

/// Writes settings to the user configuration file, creating the
/// directory on first use. Returns the path written.
///
/// Comments in an existing file are not preserved — the file is
/// regenerated from the current values.
pub fn save(settings: &Settings) -> ConfigResult<PathBuf> {
    let Some(config_dir) = dirs::config_dir() else {
        return Err(ConfigError::InvalidValue {
            key: "config_dir",
            reason: "no user configuration directory on this system".to_owned(),
        });
    };
    let dir = config_dir.join("netspd");
    let path = dir.join(FILE_NAME);
    let serialized = toml::to_string_pretty(settings).map_err(|err| ConfigError::InvalidValue {
        key: "settings",
        reason: err.to_string(),
    })?;
    std::fs::create_dir_all(&dir).map_err(|source| ConfigError::Io {
        path: dir.clone(),
        source,
    })?;
    std::fs::write(&path, serialized).map_err(|source| ConfigError::Io {
        path: path.clone(),
        source,
    })?;
    Ok(path)
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
