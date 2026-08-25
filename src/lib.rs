use std::{
    fs, io,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use serde::{Serialize, de::DeserializeOwned};

mod error;
mod shared;

pub use error::*;
pub use shared::*;

/// Target directory of the configuration files
pub enum ConfigDirectory {
    /// System default configuration directory (`~/.config/app_name`)
    #[cfg(feature = "system-dirs")]
    System(&'static str),

    /// Custom file path
    Custom(PathBuf),
}

#[derive(Debug, Clone)]
pub struct Config {
    /// Filename of the current configuration file (default `config.toml`)
    pub file: PathBuf,
    /// Directory that contains the configuration files
    pub path: PathBuf,
}

impl Config {
    pub fn new(dir: ConfigDirectory) -> Result<Self, ConfigError> {
        let config_path = match dir {
            #[cfg(feature = "system-dirs")]
            ConfigDirectory::System(app_name) => dirs::config_dir()
                .map(|p| p.join(app_name))
                .ok_or(ConfigError::SystemConfigNotFound)?,

            ConfigDirectory::Custom(config_path) => config_path,
        };

        if !config_path.is_dir() {
            fs::create_dir_all(&config_path).map_err(|err| ConfigError::Io {
                path: config_path.clone(),
                source: err,
            })?;
        }

        Ok(Self {
            file: PathBuf::from("config.toml"),
            path: config_path,
        })
    }

    /// Changes the current configuration file
    pub fn set_file(&mut self, file: impl Into<PathBuf>) -> &mut Self {
        self.file = file.into();
        self
    }

    /// Builder pattern to set the current configuration file
    pub fn with_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.file = file.into();
        self
    }

    /// Reads and deserializes the configuration file into type `T`
    pub fn read<T: DeserializeOwned>(&self) -> Result<T, ConfigError> {
        let content = fs::read_to_string(&self.file).map_err(|err| ConfigError::Io {
            path: self.file.clone(),
            source: err,
        })?;

        Ok(toml::from_str::<T>(&content)?)
    }

    /// Reads and deserializes the configuration file into type `T`. If missing config is written
    /// and returns `T::default()`
    pub fn read_or_default<T: Serialize + DeserializeOwned + Default>(
        &self,
    ) -> Result<T, ConfigError> {
        match self.read::<T>() {
            Ok(data) => Ok(data),
            Err(ConfigError::Io { source, .. }) if source.kind() == io::ErrorKind::NotFound => {
                let default_conf = T::default();
                self.write(&default_conf)?;
                Ok(default_conf)
            }
            Err(err) => Err(err),
        }
    }

    /// Serializes and writes data `T` to disk as pretty TOML file.
    pub fn write<T: Serialize>(&self, data: &T) -> Result<(), ConfigError> {
        if let Some(parent) = &self.file.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent).map_err(|err| ConfigError::Io {
                path: parent.to_path_buf(),
                source: err,
            })?;
        }

        let content = toml::to_string_pretty(data)?;
        fs::write(&self.file, content).map_err(|err| ConfigError::Io {
            path: self.file.clone(),
            source: err,
        })?;

        Ok(())
    }

    /// Loads the config and stores it in a thread safe `SharedConfig`
    pub fn load_shared<T: Serialize + DeserializeOwned>(
        self,
    ) -> Result<SharedConfig<T>, ConfigError> {
        let data = self.read::<T>()?;
        Ok(SharedConfig {
            data: Arc::new(RwLock::new(data)),
            storage: Arc::new(self),
            on_reload: None,
        })
    }

    /// Loads the config, writes and returns `T::default()` if missing, and stores it in a thread safe `SharedConfig`
    pub fn load_shared_or_default<T: Serialize + DeserializeOwned + Default>(
        self,
    ) -> Result<SharedConfig<T>, ConfigError> {
        let data = match self.read::<T>() {
            Ok(data) => data,
            Err(ConfigError::Io { source, .. }) if source.kind() == io::ErrorKind::NotFound => {
                let default_conf = T::default();
                self.write(&default_conf)?;
                default_conf
            }
            Err(err) => return Err(err),
        };

        Ok(SharedConfig {
            data: Arc::new(RwLock::new(data)),
            storage: Arc::new(self),
            on_reload: None,
        })
    }
}
