use std::time::SystemTime;

use super::{AppConfig, AppState};

impl AppState {
    #[rustfmt::skip]
    pub fn database_url(&self) -> Option<String> { self.option(|c| c.database_url.clone()) }
    #[rustfmt::skip]
    pub fn database_pool(&self) -> Option<lkjmc_store::pool::Pool> { self.option(|c| c.database_pool.clone()) }
    #[rustfmt::skip]
    pub fn database_connection(&self) -> Result<lkjmc_store::pool::PooledConnection, String> { self.database_pool().ok_or_else(|| "Database URL is not configured".to_string())?.get().map_err(|error| error.to_string()) }
    #[rustfmt::skip]
    pub fn database_pool_size(&self) -> u32 { self.config.read().map(|c| c.database_pool_size).unwrap_or(8) }
    #[rustfmt::skip]
    pub fn config_path(&self) -> Option<String> { self.option(|c| c.config_path.clone()) }
    #[rustfmt::skip]
    pub fn config_root(&self) -> String { self.value(|c| c.config_root.clone()) }
    #[rustfmt::skip]
    pub fn log_root(&self) -> String { self.value(|c| c.log_root.clone()) }
    #[rustfmt::skip]
    pub fn jar_root(&self) -> String { self.value(|c| c.jar_root.clone()) }

    pub fn asset_root(&self) -> String {
        let jar_root = self.jar_root();
        jar_root
            .strip_suffix("/jars")
            .map(|root| format!("{root}/assets"))
            .unwrap_or(jar_root)
    }

    #[rustfmt::skip]
    pub fn data_root(&self) -> String { self.value(|c| c.data_root.clone()) }
    #[rustfmt::skip]
    pub fn socket_path(&self) -> String { self.value(|c| c.socket_path.clone()) }
    #[rustfmt::skip]
    pub fn http_listener(&self) -> Option<String> { self.option(|c| c.http_listener.clone()) }
    #[rustfmt::skip]
    pub fn http_token_file(&self) -> Option<String> { self.option(|c| c.http_token_file.clone()) }
    #[rustfmt::skip]
    pub fn reconciler_enabled(&self) -> bool { self.config.read().map(|c| c.reconciler_enabled).unwrap_or(false) }
    #[rustfmt::skip]
    pub fn started_at(&self) -> SystemTime { self.config.read().map(|c| c.started_at).unwrap_or(SystemTime::UNIX_EPOCH) }

    fn value(&self, reader: impl FnOnce(&AppConfig) -> String) -> String {
        self.config
            .read()
            .map(|config| reader(&config))
            .unwrap_or_default()
    }

    pub(super) fn option<T>(&self, reader: impl FnOnce(&AppConfig) -> Option<T>) -> Option<T> {
        self.config.read().ok().and_then(|config| reader(&config))
    }
}
