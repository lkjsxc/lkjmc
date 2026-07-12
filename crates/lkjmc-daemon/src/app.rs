mod http_tokens;
mod unix_peers;

use std::sync::{Arc, Mutex, RwLock};
use std::time::SystemTime;

use lkjmc_core::config::LkjmcConfig;

use crate::runtime::local::LocalRuntime;
use crate::runtime::{RuntimeAdapter, RuntimeCapabilities};

#[derive(Clone)]
pub struct AppState {
    pub runtime: Arc<Mutex<Box<dyn RuntimeAdapter>>>,
    pub web_sessions: crate::web::sessions::WebSessions,
    pub credential_cache: crate::credential_cache::CredentialCache,
    secrets: crate::support::secret_provider::SecretProvider,
    config: Arc<RwLock<AppConfig>>,
}

#[derive(Clone)]
struct AppConfig {
    database_url: Option<String>,
    database_pool: Option<lkjmc_store::pool::Pool>,
    database_pool_size: u32,
    config_root: String,
    log_root: String,
    jar_root: String,
    data_root: String,
    config_path: Option<String>,
    socket_path: String,
    http_listener: Option<String>,
    http_token_file: Option<String>,
    reconciler_enabled: bool,
    unix_peer_policy: Option<crate::transport::peer::UnixPeerPolicy>,
    started_at: SystemTime,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn with_config_path(
        database_url: Option<String>,
        database_pool_size: u32,
        config_root: String,
        log_root: String,
        jar_root: String,
        data_root: String,
        config_path: Option<String>,
        http_token_file: Option<String>,
        http_token: Option<String>,
    ) -> Self {
        let database_pool = database_url
            .as_deref()
            .and_then(|url| lkjmc_store::pool::build(url, database_pool_size).ok());
        Self {
            runtime: Arc::new(Mutex::new(Box::new(LocalRuntime::new()))),
            credential_cache: crate::credential_cache::CredentialCache::default(),
            secrets: crate::support::secret_provider::SecretProvider::new(http_token),
            config: Arc::new(RwLock::new(AppConfig {
                database_url,
                database_pool,
                database_pool_size,
                config_root,
                log_root,
                jar_root,
                data_root,
                config_path,
                socket_path: "/run/lkjmc/daemon.sock".to_string(),
                http_listener: None,
                http_token_file,
                reconciler_enabled: false,
                unix_peer_policy: None,
                started_at: SystemTime::now(),
            })),
            web_sessions: crate::web::sessions::WebSessions::new(),
        }
    }

    pub fn with_runtime_metadata(
        &self,
        socket_path: String,
        http_listener: Option<String>,
        reconciler_enabled: bool,
    ) -> Result<(), String> {
        let mut config = self
            .config
            .write()
            .map_err(|_| "config lock poisoned".to_string())?;
        config.socket_path = socket_path;
        config.http_listener = http_listener;
        config.reconciler_enabled = reconciler_enabled;
        Ok(())
    }
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

    fn value(&self, reader: impl FnOnce(&AppConfig) -> String) -> String {
        self.config
            .read()
            .map(|config| reader(&config))
            .unwrap_or_default()
    }

    fn option<T>(&self, reader: impl FnOnce(&AppConfig) -> Option<T>) -> Option<T> {
        self.config.read().ok().and_then(|config| reader(&config))
    }

    #[rustfmt::skip]
    pub fn reconciler_enabled(&self) -> bool { self.config.read().map(|c| c.reconciler_enabled).unwrap_or(false) }
    #[rustfmt::skip]
    pub fn started_at(&self) -> SystemTime { self.config.read().map(|c| c.started_at).unwrap_or(SystemTime::UNIX_EPOCH) }

    pub fn set_runtime(&self, runtime: Box<dyn RuntimeAdapter>) -> Result<(), String> {
        *self
            .runtime
            .lock()
            .map_err(|_| "runtime lock poisoned".to_string())? = runtime;
        Ok(())
    }

    pub fn runtime_adapter_name(&self) -> Result<&'static str, String> {
        self.runtime
            .lock()
            .map(|runtime| runtime.name())
            .map_err(|_| "runtime lock poisoned".to_string())
    }

    pub fn runtime_capabilities(&self) -> Result<RuntimeCapabilities, String> {
        self.runtime
            .lock()
            .map(|runtime| runtime.capabilities())
            .map_err(|_| "runtime lock poisoned".to_string())
    }

    pub fn runtime_config(&self) -> Result<Option<LkjmcConfig>, String> {
        match self.config_path() {
            Some(path) => crate::support::daemon_config::read_config(&path).map(Some),
            None => Ok(None),
        }
    }

    pub fn reload_from_file(&self, path: &str) -> Result<(), String> {
        let loaded = crate::support::daemon_config::load(path)?;
        let mut config = self
            .config
            .write()
            .map_err(|_| "config lock poisoned".to_string())?;
        config.database_pool = Some(
            lkjmc_store::pool::build(&loaded.database_url, loaded.database_pool_size)
                .map_err(|error| error.to_string())?,
        );
        config.database_pool_size = loaded.database_pool_size;
        config.database_url = Some(loaded.database_url);
        config.config_root = loaded.config_root;
        config.log_root = loaded.log_root;
        config.jar_root = loaded.jar_root;
        config.data_root = loaded.data_root;
        config.config_path = Some(path.to_string());
        config.http_token_file = Some(loaded.http_token_file);
        Ok(())
    }
}
