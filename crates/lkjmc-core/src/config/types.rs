use super::runtime_types::RuntimeConfig;
use crate::instance::{DesiredState, InstanceKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
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
    #[serde(default = "super::defaults::daemon_http")]
    pub daemon_http: DaemonHttpConfig,
    #[serde(default = "super::defaults::assets")]
    pub assets: AssetsConfig,
    #[serde(default = "super::defaults::plugins")]
    pub plugins: PluginsConfig,
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
    #[serde(default = "super::defaults::database_pool_size")]
    pub pool_size: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkConfig {
    #[serde(default = "super::defaults::network_name")]
    pub name: String,
    pub default_locale: String,
    pub fallback_server: String,
    pub online_mode: bool,
    pub velocity_forwarding: VelocityForwarding,
    #[serde(default = "super::defaults::forwarding_secret_file")]
    pub forwarding_secret_file: String,
    #[serde(default = "super::defaults::java_entry")]
    pub java_entry: JavaEntry,
    #[serde(default = "super::defaults::bedrock_entry")]
    pub bedrock_entry: BedrockEntry,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VelocityForwarding {
    Modern,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JavaEntry {
    pub bind_host: String,
    pub port: u16,
    #[serde(default)]
    pub public_hosts: Vec<String>,
    #[serde(default)]
    pub preferred_public_host: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BedrockEntry {
    pub mode: BedrockMode,
    pub host: String,
    pub port: u16,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BedrockMode {
    Auto,
    Enabled,
    Disabled,
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
pub struct DaemonHttpConfig {
    pub enabled: bool,
    pub address: String,
    pub token_file: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssetsConfig {
    pub root: String,
    pub server_channel: String,
    pub plugin_channel: String,
    pub user_agent: String,
    pub download_timeout_seconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginsConfig {
    #[serde(default = "super::defaults::lkjmc_plugin")]
    pub lkjmc: LkjmcPluginConfig,
    #[serde(default = "super::defaults::backend_plugin")]
    pub viaversion: PluginInstallConfig,
    #[serde(default = "super::defaults::backend_plugin")]
    pub viabackwards: PluginInstallConfig,
    #[serde(default = "super::defaults::proxy_plugin")]
    pub geyser: PluginInstallConfig,
    #[serde(default = "super::defaults::floodgate_plugin")]
    pub floodgate: FloodgatePluginConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LkjmcPluginConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginInstallConfig {
    pub mode: PluginMode,
    pub install_on: PluginInstallTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FloodgatePluginConfig {
    pub mode: PluginMode,
    pub install_on: PluginInstallTarget,
    pub backend_api: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginMode {
    Enabled,
    Disabled,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginInstallTarget {
    Backend,
    Proxy,
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
