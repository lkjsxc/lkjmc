use serde::{Deserialize, Serialize};

use crate::config::JavaEntry;
use crate::id::InstanceId;
use crate::instance::InstanceKind;

use super::plugin::PluginId;
use super::BootstrapRuntimeSettings;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesiredNetwork {
    pub proxy: DesiredInstance,
    pub backends: Vec<DesiredInstance>,
    pub forwarding: ForwardingPlan,
    pub daemon_http: DaemonHttpPlan,
    pub plugin_set: DesiredPluginSet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesiredInstance {
    pub id: InstanceId,
    pub kind: InstanceKind,
    pub server_port: u16,
    pub memory_mb: u32,
    pub template: String,
    pub bind_host: String,
    pub public_hosts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForwardingPlan {
    pub mode: ForwardingMode,
    pub online_mode: bool,
    pub secret_file: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ForwardingMode {
    Modern,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonHttpPlan {
    pub enabled: bool,
    pub address: String,
    pub token_file: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesiredPluginSet {
    pub required: Vec<PluginId>,
    pub optional: Vec<PluginId>,
}

impl DesiredNetwork {
    pub fn playable(
        java_entry: &JavaEntry,
        backend_port: u16,
        runtime: &BootstrapRuntimeSettings,
    ) -> Self {
        Self {
            proxy: DesiredInstance {
                id: InstanceId::internal("proxy"),
                kind: InstanceKind::Velocity,
                server_port: java_entry.port,
                memory_mb: runtime.proxy_memory_mb,
                template: "velocity-modern".to_string(),
                bind_host: java_entry.bind_host.clone(),
                public_hosts: java_entry.public_hosts.clone(),
            },
            backends: vec![DesiredInstance {
                id: InstanceId::internal("hub"),
                kind: InstanceKind::Folia,
                server_port: backend_port,
                memory_mb: runtime.backend_memory_mb,
                template: "folia-survival".to_string(),
                bind_host: "127.0.0.1".to_string(),
                public_hosts: Vec::new(),
            }],
            forwarding: ForwardingPlan {
                mode: ForwardingMode::Modern,
                online_mode: true,
                secret_file: runtime.forwarding_secret_file.clone(),
            },
            daemon_http: DaemonHttpPlan {
                enabled: runtime.daemon_http_enabled,
                address: runtime.daemon_http_address.clone(),
                token_file: runtime.daemon_http_token_file.clone(),
            },
            plugin_set: DesiredPluginSet {
                required: vec![PluginId::LkjmcPaper, PluginId::LkjmcVelocity],
                optional: vec![
                    PluginId::ViaVersion,
                    PluginId::ViaBackwards,
                    PluginId::Geyser,
                    PluginId::Floodgate,
                ],
            },
        }
    }
}
