use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::ConfigError;
use crate::instance::{DesiredState, InstanceKind};
use crate::validation::{is_absolute_path, is_kebab_id, is_non_empty, is_valid_port};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LkjmcConfig {
    pub install_root: String,
    pub config_root: String,
    pub data_root: String,
    pub log_root: String,
    pub socket_path: String,
    pub database: DatabaseConfig,
    pub network: NetworkConfig,
    pub jars: JarsConfig,
    pub runtime: RuntimeConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub secret_file: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkConfig {
    pub default_locale: String,
    pub fallback_server: String,
    pub online_mode: bool,
    pub velocity_forwarding: VelocityForwarding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VelocityForwarding {
    Modern,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JarsConfig {
    pub root: String,
    pub default_channel: String,
    pub user_agent: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeConfig {
    pub adapter: RuntimeAdapter,
    pub default_java_memory_mb: u32,
    pub stop_timeout_seconds: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeAdapter {
    LocalProcess,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstanceFileConfig {
    pub id: String,
    pub kind: InstanceKind,
    pub desired_state: DesiredState,
    pub jar_ref: String,
    pub server_port: u16,
    pub rcon_port: Option<u16>,
    pub memory_mb: u32,
    pub template: String,
    pub properties: BTreeMap<String, Value>,
    pub plugins: BTreeMap<String, bool>,
    pub sync: InstanceSyncConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstanceSyncConfig {
    pub player_profile: bool,
    pub location: bool,
}

impl LkjmcConfig {
    pub fn from_json_str(input: &str) -> Result<Self, ConfigError> {
        let config: Self =
            serde_json::from_str(input).map_err(|error| ConfigError::json(error.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        require_path("installRoot", &self.install_root)?;
        require_path("configRoot", &self.config_root)?;
        require_path("dataRoot", &self.data_root)?;
        require_path("logRoot", &self.log_root)?;
        require_path("socketPath", &self.socket_path)?;
        require_non_empty("database.host", &self.database.host)?;
        require_non_empty("database.database", &self.database.database)?;
        require_non_empty("database.user", &self.database.user)?;
        require_path("database.secretFile", &self.database.secret_file)?;
        if !is_valid_port(self.database.port) {
            return Err(ConfigError::invalid("database.port", "must be 1..65535"));
        }
        require_non_empty("network.defaultLocale", &self.network.default_locale)?;
        require_kebab("network.fallbackServer", &self.network.fallback_server)?;
        require_path("jars.root", &self.jars.root)?;
        require_non_empty("jars.defaultChannel", &self.jars.default_channel)?;
        if !self.jars.user_agent.contains("lkjmc") {
            return Err(ConfigError::invalid(
                "jars.userAgent",
                "must identify lkjmc",
            ));
        }
        if self.runtime.default_java_memory_mb == 0 {
            return Err(ConfigError::invalid(
                "runtime.defaultJavaMemoryMb",
                "must be positive",
            ));
        }
        if self.runtime.stop_timeout_seconds == 0 {
            return Err(ConfigError::invalid(
                "runtime.stopTimeoutSeconds",
                "must be positive",
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
        if !is_valid_port(self.server_port) {
            return Err(ConfigError::invalid("serverPort", "must be 1..65535"));
        }
        if self.rcon_port.is_some_and(|port| !is_valid_port(port)) {
            return Err(ConfigError::invalid("rconPort", "must be 1..65535"));
        }
        if self.memory_mb == 0 {
            return Err(ConfigError::invalid("memoryMb", "must be positive"));
        }
        Ok(())
    }
}

fn require_path(field: &'static str, value: &str) -> Result<(), ConfigError> {
    if is_absolute_path(value) {
        Ok(())
    } else {
        Err(ConfigError::invalid(field, "must be an absolute path"))
    }
}

fn require_kebab(field: &'static str, value: &str) -> Result<(), ConfigError> {
    if is_kebab_id(value) {
        Ok(())
    } else {
        Err(ConfigError::invalid(field, "must be lowercase kebab-case"))
    }
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), ConfigError> {
    if is_non_empty(value) {
        Ok(())
    } else {
        Err(ConfigError::invalid(field, "must not be empty"))
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;
