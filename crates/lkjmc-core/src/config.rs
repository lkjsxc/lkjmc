mod defaults;
mod types;
mod validate;

pub use types::*;

use validate::{
    require_kebab, require_non_empty, require_path, require_port, require_positive,
    require_user_agent,
};

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
        self.validate_runtime()?;
        Ok(())
    }

    fn validate_paths(&self) -> Result<(), ConfigError> {
        require_path("installRoot", &self.install_root)?;
        require_path("configRoot", &self.config_root)?;
        require_path("dataRoot", &self.data_root)?;
        require_path("logRoot", &self.log_root)?;
        require_path("socketPath", &self.socket_path)?;
        require_path(
            "network.forwardingSecretFile",
            &self.network.forwarding_secret_file,
        )?;
        require_path("jars.root", &self.jars.root)?;
        require_path("assets.root", &self.assets.root)?;
        require_path("daemonHttp.tokenFile", &self.daemon_http.token_file)
    }

    fn validate_database(&self) -> Result<(), ConfigError> {
        require_non_empty("database.host", &self.database.host)?;
        require_non_empty("database.database", &self.database.database)?;
        require_non_empty("database.user", &self.database.user)?;
        require_path("database.secretFile", &self.database.secret_file)?;
        require_port("database.port", self.database.port)
    }

    fn validate_network(&self) -> Result<(), ConfigError> {
        require_kebab("network.name", &self.network.name)?;
        require_non_empty("network.defaultLocale", &self.network.default_locale)?;
        require_kebab("network.fallbackServer", &self.network.fallback_server)?;
        require_non_empty("network.javaEntry.host", &self.network.java_entry.host)?;
        require_port("network.javaEntry.port", self.network.java_entry.port)?;
        require_non_empty(
            "network.bedrockEntry.host",
            &self.network.bedrock_entry.host,
        )?;
        require_port("network.bedrockEntry.port", self.network.bedrock_entry.port)?;
        if self.network.bedrock_entry.mode != BedrockMode::Disabled
            && self.network.java_entry.port == self.network.bedrock_entry.port
        {
            return Err(ConfigError::invalid(
                "network.bedrockEntry.port",
                "must differ from Java TCP port unless disabled",
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
        Ok(())
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

impl Default for JavaEntry {
    fn default() -> Self {
        defaults::java_entry()
    }
}

impl Default for BedrockEntry {
    fn default() -> Self {
        defaults::bedrock_entry()
    }
}

impl Default for PluginsConfig {
    fn default() -> Self {
        defaults::plugins()
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;
