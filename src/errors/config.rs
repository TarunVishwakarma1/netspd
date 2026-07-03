//! Errors produced while loading configuration and themes.

use std::path::PathBuf;

use thiserror::Error;

/// Convenient result alias for configuration operations.
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Failures while reading or interpreting configuration files.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// A configuration file exists but could not be read.
    #[error("failed to read {path}: {source}")]
    Io {
        /// The file that failed to read.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// A configuration file contained invalid TOML.
    #[error("failed to parse {path}: {source}")]
    Parse {
        /// The file that failed to parse.
        path: PathBuf,
        /// The underlying TOML error.
        source: Box<toml::de::Error>,
    },

    /// A theme definition contained invalid TOML.
    #[error("invalid theme definition: {0}")]
    InvalidTheme(Box<toml::de::Error>),

    /// A color value was not a valid `#rrggbb` hex string.
    #[error("invalid color value: {0:?}")]
    InvalidColor(String),

    /// A configuration value was outside its allowed range.
    #[error("invalid value for {key}: {reason}")]
    InvalidValue {
        /// The offending configuration key.
        key: &'static str,
        /// Why the value was rejected.
        reason: String,
    },
}
