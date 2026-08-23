use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock},

};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher, event::ModifyKind};

use serde::{Serialize, de::DeserializeOwned};

use crate::*;

type ReloadCallback<T> = Arc<dyn Fn(&T) + Send + Sync>;

#[derive(Clone)]
pub struct SharedConfig<T> {
    pub data: Arc<RwLock<T>>,
    pub storage: Arc<Config>,
    pub on_reload: Option<ReloadCallback<T>>,
}

impl<T: Serialize + DeserializeOwned> SharedConfig<T> {
    pub fn get<R>(&self, f: impl FnOnce(&T) -> R) -> Result<R, ConfigError> {
        let guard = self.data.read().map_err(|_| ConfigError::LockPoisoned)?;

        Ok(f(guard.deref()))
    }

    pub fn update<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, ConfigError> {
        let mut guard = self.data.write().map_err(|_| ConfigError::LockPoisoned)?;
        Ok(f(guard.deref_mut()))
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let guard = self.data.read().map_err(|_| ConfigError::LockPoisoned)?;

        self.storage.write(guard.deref())
    }

    pub fn reload(&self) -> Result<(), ConfigError> {
        let fresh_data: T = self.storage.read()?;
        {
            let mut guard = self.data.write().map_err(|_| ConfigError::LockPoisoned)?;
            *guard = fresh_data;
        }
        if let Some(ref callback) = self.on_reload {
            self.get(|d| callback(d))?;
        }

        Ok(())
    }

    pub fn on_reload(mut self, f: impl Fn(&T) + Send + Sync + 'static) -> Self {
        self.on_reload = Some(Arc::new(f));
        self
    }
}

#[cfg(feature = "watcher")]
impl<T: Serialize + DeserializeOwned + Send + Sync + 'static> SharedConfig<T> {
    pub fn spawn_watcher(self) -> Result<RecommendedWatcher, ConfigError> {
        let target = self.storage.target_path();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res
                && matches!(
                    event.kind,
                    EventKind::Modify(ModifyKind::Metadata(_)) | EventKind::Create(_)
                )
            // ModifyKind::Data fires twice sometimes, so i'll track with ModifyKind::Metadata
            // instead
            {
                let _ = self.reload();
            }
        })
        .map_err(|e| ConfigError::Io {
            path: target.clone(),
            source: io::Error::other(e.to_string()),
        })?;

        watcher
            .watch(&target, RecursiveMode::NonRecursive)
            .map_err(|e| ConfigError::Io {
                path: target,
                source: io::Error::other(e.to_string()),
            })?;

        Ok(watcher)
    }
}
