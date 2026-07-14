mod defaults;
mod entry;
mod network_intent;
mod network_validate;
mod runtime_types;
mod runtime_validate;
mod types;
mod validate;

pub use network_intent::*;
pub use runtime_types::*;
pub use types::*;
pub use validate::literal_loopback_socket;
use validate::*;

use crate::error::ConfigError;

impl LkjmcConfig {
    pub fn from_json_str(input: &str) -> Result<Self, ConfigError> {
        let config: Self =
            serde_json::from_str(input).map_err(|error| ConfigError::json(error.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        self.validate_paths()?;
        self.validate_database()?;
        self.validate_network()?;
        self.validate_assets()?;
        if self.daemon_http.enabled {
            require_loopback_address("daemonHttp.address", &self.daemon_http.address)?;
        }
        self.validate_runtime()?;
        Ok(())
    }

    fn validate_paths(&self) -> Result<(), ConfigError> {
        require_path("installRoot", &self.install_root)?;
        require_path("configRoot", &self.config_root)?;
        require_path("dataRoot", &self.data_root)?;
        require_path("logRoot", &self.log_root)?;
        require_path("socketPath", &self.socket_path)?;
        require_path("jars.root", &self.jars.root)?;
        require_path("assets.root", &self.assets.root)?;
        require_path("daemonHttp.tokenFile", &self.daemon_http.token_file)
    }

    fn validate_database(&self) -> Result<(), ConfigError> {
        require_non_empty("database.host", &self.database.host)?;
        require_non_empty("database.database", &self.database.database)?;
        require_non_empty("database.user", &self.database.user)?;
        require_path("database.secretFile", &self.database.secret_file)?;
        require_range("database.poolSize", self.database.pool_size, 1, 64)?;
        require_port("database.port", self.database.port)
    }

    fn validate_network(&self) -> Result<(), ConfigError> {
        self.network.validate()?;
        let selected = match self.runtime.adapter {
            RuntimeAdapter::LocalProcess => NetworkRuntime::LocalProcess,
            RuntimeAdapter::Kubernetes => NetworkRuntime::Kubernetes,
        };
        if selected != self.network.capabilities.runtime {
            return Err(ConfigError::invalid(
                "network.capabilities.runtime",
                "must match runtime.adapter",
            ));
        }
        Ok(())
    }

    fn validate_assets(&self) -> Result<(), ConfigError> {
        require_non_empty("jars.defaultChannel", &self.jars.default_channel)?;
        require_user_agent("jars.userAgent", &self.jars.user_agent)?;
        require_non_empty("assets.serverChannel", &self.assets.server_channel)?;
        require_non_empty("assets.pluginChannel", &self.assets.plugin_channel)?;
        require_user_agent("assets.userAgent", &self.assets.user_agent)?;
        if self.assets.download_timeout_seconds == 0 {
            return Err(ConfigError::invalid(
                "assets.downloadTimeoutSeconds",
                "must be positive",
            ));
        }
        Ok(())
    }

    fn validate_runtime(&self) -> Result<(), ConfigError> {
        require_positive(
            "runtime.defaultJavaMemoryMb",
            self.runtime.default_java_memory_mb,
        )?;
        require_positive(
            "runtime.proxyJavaMemoryMb",
            self.runtime.proxy_java_memory_mb,
        )?;
        require_positive(
            "runtime.stopTimeoutSeconds",
            self.runtime.stop_timeout_seconds,
        )?;
        require_port("runtime.portRangeStart", self.runtime.port_range_start)?;
        require_port("runtime.portRangeEnd", self.runtime.port_range_end)?;
        if self.runtime.port_range_start > self.runtime.port_range_end {
            return Err(ConfigError::invalid(
                "runtime.portRangeStart",
                "must be less than or equal to portRangeEnd",
            ));
        }
        runtime_validate::validate_kubernetes_runtime(&self.runtime)
    }
}

impl InstanceFileConfig {
    pub fn from_json_str(input: &str) -> Result<Self, ConfigError> {
        let config: Self =
            serde_json::from_str(input).map_err(|error| ConfigError::json(error.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        require_kebab("id", &self.id)?;
        require_non_empty("jarRef", &self.jar_ref)?;
        require_non_empty("template", &self.template)?;
        require_port("serverPort", self.server_port)?;
        if let Some(port) = self.rcon_port {
            require_port("rconPort", port)?;
        }
        require_positive("memoryMb", self.memory_mb)
    }
}

#[cfg(test)]
#[path = "config_listener_tests.rs"]
mod config_listener_tests;
#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;
