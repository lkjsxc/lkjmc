use std::sync::{Arc, Mutex, RwLock};

use crate::runtime_local::LocalRuntime;

#[derive(Clone)]
pub struct AppState {
    pub runtime: Arc<Mutex<LocalRuntime>>,
    config: Arc<RwLock<AppConfig>>,
}

#[derive(Clone)]
struct AppConfig {
    database_url: Option<String>,
    config_root: String,
    log_root: String,
    jar_root: String,
    data_root: String,
    config_path: Option<String>,
}

impl AppState {
    pub fn with_config_path(
        database_url: Option<String>,
        config_root: String,
        log_root: String,
        jar_root: String,
        data_root: String,
        config_path: Option<String>,
    ) -> Self {
        Self {
            runtime: Arc::new(Mutex::new(LocalRuntime::new())),
            config: Arc::new(RwLock::new(AppConfig {
                database_url,
                config_root,
                log_root,
                jar_root,
                data_root,
                config_path,
            })),
        }
    }

    pub fn database_url(&self) -> Option<String> {
        self.config
            .read()
            .ok()
            .and_then(|config| config.database_url.clone())
    }

    pub fn config_path(&self) -> Option<String> {
        self.config
            .read()
            .ok()
            .and_then(|config| config.config_path.clone())
    }

    pub fn config_root(&self) -> String {
        self.config
            .read()
            .map(|config| config.config_root.clone())
            .unwrap_or_default()
    }

    pub fn log_root(&self) -> String {
        self.config
            .read()
            .map(|config| config.log_root.clone())
            .unwrap_or_default()
    }

    pub fn jar_root(&self) -> String {
        self.config
            .read()
            .map(|config| config.jar_root.clone())
            .unwrap_or_default()
    }

    pub fn data_root(&self) -> String {
        self.config
            .read()
            .map(|config| config.data_root.clone())
            .unwrap_or_default()
    }

    pub fn reload_from_file(&self, path: &str) -> Result<(), String> {
        let loaded = crate::daemon_config::load(path)?;
        let mut config = self
            .config
            .write()
            .map_err(|_| "config lock poisoned".to_string())?;
        config.database_url = Some(loaded.database_url);
        config.config_root = loaded.config_root;
        config.log_root = loaded.log_root;
        config.jar_root = loaded.jar_root;
        config.data_root = loaded.data_root;
        config.config_path = Some(path.to_string());
        Ok(())
    }
}
