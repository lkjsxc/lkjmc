use std::path::PathBuf;
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
        if let Ok(Some(config)) = self.runtime_config() {
            return config.assets.root;
        }
        let jar_root = self.jar_root();
        jar_root
            .strip_suffix("/jars")
            .map(|root| format!("{root}/assets"))
            .unwrap_or(jar_root)
    }

    #[rustfmt::skip]
    pub fn data_root(&self) -> String { self.value(|c| c.data_root.clone()) }

    pub fn plugin_credential_root(&self) -> Result<PathBuf, String> {
        let instances_root = PathBuf::from(self.data_root());
        if !instances_root.is_absolute() {
            return Err("managed instance root is not absolute".to_string());
        }
        let product_root = instances_root
            .parent()
            .ok_or_else(|| "managed instance root has no parent".to_string())?;
        Ok(product_root.join("private/plugin-credentials"))
    }

    pub fn plugin_credential_path(&self, instance_id: &str) -> Result<String, String> {
        let id = lkjmc_core::id::InstanceId::parse(instance_id.to_string())
            .map_err(|error| error.to_string())?;
        self.plugin_credential_root()?
            .join(format!("{}.secret", id.as_str()))
            .to_str()
            .map(ToString::to_string)
            .ok_or_else(|| "plugin credential path is not UTF-8".to_string())
    }
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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::AppState;

    #[test]
    fn configured_asset_root_is_not_derived_from_the_jar_root() -> Result<(), String> {
        let root = std::env::temp_dir().join(format!(
            "lkjmc-configured-asset-root-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let config_path = root.join("lkjmc.json");
        let config = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../config/defaults/daemon.json.example"
        ))
        .replace("/opt/lkjmc/assets", "/opt/lkjmc/runtime-assets");
        fs::write(&config_path, config).map_err(|error| error.to_string())?;
        let state = AppState::with_config_path(
            None,
            8,
            "/etc/lkjmc".to_string(),
            "/var/log/lkjmc/instances".to_string(),
            "/different/jars".to_string(),
            "/var/lib/lkjmc/instances".to_string(),
            Some(config_path.display().to_string()),
            None,
            None,
        );
        assert_eq!(state.asset_root(), "/opt/lkjmc/runtime-assets");
        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }
}
