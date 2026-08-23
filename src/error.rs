use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Failed to deserialize config data: {0}")]
    Deserialization(#[from] toml::de::Error),

    #[error("Failed to serialize config data: {0}")]
    Serialization(#[from] toml::ser::Error),

    #[error("No valid system config file found")]
    SystemConfigNotFound,

    #[error("Lock poisoned while accessing config data")]
    LockPoisoned,
}
