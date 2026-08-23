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

pub enum ConfigDirectory {
    #[cfg(feature = "system-dirs")]
    System(&'static str),

    Custom(PathBuf),
}

#[derive(Debug, Clone)]
pub struct Config {
    pub file: PathBuf,
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

    pub fn set_file(&mut self, file: impl Into<PathBuf>) -> &mut Self {
        self.file = file.into();
        self
    }

    pub fn with_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.file = file.into();
        self
    }

    pub fn target_path(&self) -> PathBuf {
        self.path.join(&self.file)
    }

    pub fn read<T: DeserializeOwned>(&self) -> Result<T, ConfigError> {
        let file_path = self.target_path();

        let content = fs::read_to_string(&file_path).map_err(|err| ConfigError::Io {
            path: file_path,
            source: err,
        })?;

        Ok(toml::from_str::<T>(&content)?)
    }

    pub fn read_file<T: DeserializeOwned>(
        &self,
        file: impl Into<PathBuf>,
    ) -> Result<T, ConfigError> {
        self.clone().with_file(file).read()
    }

    pub fn write<T: Serialize>(&self, data: &T) -> Result<(), ConfigError> {
        let file = self.target_path();

        if let Some(parent) = file.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent).map_err(|err| ConfigError::Io {
                path: parent.to_path_buf(),
                source: err,
            })?;
        }

        let content = toml::to_string_pretty(data)?;
        fs::write(&file, content).map_err(|err| ConfigError::Io {
            path: file,
            source: err,
        })?;

        Ok(())
    }

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
