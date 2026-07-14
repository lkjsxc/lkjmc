mod admission;
mod config_access;
mod database;
mod http_tokens;
mod unix_peers;

use std::sync::{Arc, RwLock};
#[cfg(test)]
use std::time::Duration;
use std::time::SystemTime;

use lkjmc_core::config::LkjmcConfig;

#[cfg(test)]
pub(crate) use admission::Admission;
pub(crate) use admission::{BlockingError, RequestAdmission};

use crate::runtime::local::LocalRuntime;
use crate::runtime::RuntimeAdapter;

#[derive(Clone)]
pub struct AppState {
    runtime: Arc<dyn RuntimeAdapter>,
    lifecycle: crate::runtime::LifecycleCoordinator,
    pub web_sessions: crate::web::sessions::WebSessions,
    pub credential_cache: crate::credential_cache::CredentialCache,
    secrets: crate::support::secret_provider::SecretProvider,
    config: Arc<RwLock<AppConfig>>,
    request_admission: admission::Admission,
    maintenance: crate::maintenance::Maintenance,
    metrics: crate::observability::metrics::Metrics,
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
    #[cfg(test)]
    test_lock_timeout: Option<Duration>,
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
        let database_pool = database_url.as_deref().and_then(|url| {
            lkjmc_store::pool::build(url, database_pool_size, crate::command_lifecycle::DEADLINE)
                .ok()
        });
        Self {
            runtime: Arc::new(LocalRuntime::with_data_root(&data_root)),
            lifecycle: crate::runtime::LifecycleCoordinator::new(),
            credential_cache: crate::credential_cache::CredentialCache::default(),
            secrets: crate::support::secret_provider::SecretProvider::new(http_token),
            request_admission: admission::Admission::new(),
            maintenance: crate::maintenance::Maintenance::default(),
            metrics: crate::observability::metrics::Metrics::default(),
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
                #[cfg(test)]
                test_lock_timeout: None,
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
    pub fn with_runtime(mut self, runtime: Arc<dyn RuntimeAdapter>) -> Result<Self, String> {
        runtime.check_capabilities()?;
        self.runtime = runtime;
        Ok(self)
    }

    pub fn runtime(&self) -> Arc<dyn RuntimeAdapter> {
        Arc::clone(&self.runtime)
    }

    pub fn coordinate_runtime<T>(
        &self,
        id: &str,
        work: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        self.lifecycle.run(id, work)
    }

    pub fn runtime_adapter_name(&self) -> &'static str {
        self.runtime.name()
    }

    pub fn runtime_capabilities(&self) -> crate::runtime::RuntimeCapabilities {
        self.runtime.capabilities()
    }

    pub fn shutdown_runtime(&self) -> Result<(), String> {
        self.lifecycle.close();
        self.runtime
            .runtime_shutdown(std::time::Duration::from_secs(8))
    }

    pub fn runtime_config(&self) -> Result<Option<LkjmcConfig>, String> {
        match self.config_path() {
            Some(path) => crate::support::daemon_config::read_config(&path).map(Some),
            None => Ok(None),
        }
    }

    pub(crate) fn admit_request(&self) -> Option<RequestAdmission> {
        self.request_admission.try_admit()
    }

    pub(crate) fn stop_admission(&self) {
        self.request_admission.close();
    }

    pub(crate) async fn wait_for_admitted_work(&self) -> Result<(), String> {
        self.request_admission
            .wait_for_idle()
            .await
            .map_err(|_| "admitted request worker join failed".to_string())
    }

    pub(crate) fn start_maintenance(&self) -> Result<(), String> {
        self.maintenance.start(self.database_pool())
    }

    pub(crate) async fn shutdown_maintenance(&self) -> Result<(), String> {
        self.maintenance.shutdown().await
    }

    pub(crate) fn maintenance_diagnostics(&self) -> crate::maintenance::Diagnostics {
        self.maintenance.diagnostics()
    }

    pub(crate) fn metrics(&self) -> &crate::observability::metrics::Metrics {
        &self.metrics
    }

    pub(crate) fn admission_diagnostics(&self) -> (bool, usize) {
        self.request_admission.diagnostics()
    }
}
