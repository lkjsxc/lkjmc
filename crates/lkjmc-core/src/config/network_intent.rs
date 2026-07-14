use crate::instance::{DesiredState, InstanceKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkConfig {
    pub revision: u64,
    pub instances: Vec<NetworkInstance>,
    pub routes: Vec<NetworkRoute>,
    pub listeners: Vec<NetworkListener>,
    pub auth: NetworkAuth,
    pub forwarding: NetworkForwarding,
    pub assets: Vec<NetworkAsset>,
    pub capabilities: NetworkCapabilities,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkInstance {
    pub id: String,
    pub owner: NetworkOwner,
    pub kind: InstanceKind,
    pub desired_state: DesiredState,
    pub listener: String,
    pub memory_mb: u32,
    pub asset_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkRoute {
    pub id: String,
    pub listener: String,
    pub target: String,
    pub fallbacks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkListener {
    pub id: String,
    pub protocol: ListenerProtocol,
    pub bind_host: String,
    pub port: u16,
    pub public_hosts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkAuth {
    pub online_mode: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkForwarding {
    pub mode: ForwardingMode,
    pub secret_file: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkAsset {
    pub id: String,
    pub kind: AssetKind,
    pub path: String,
    pub sha256: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkCapabilities {
    pub runtime: NetworkRuntime,
    pub mounted_config: bool,
    pub mounted_secrets: bool,
    pub mounted_assets: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkOwner {
    LkjmcDaemon,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ListenerProtocol {
    JavaTcp,
    BedrockUdp,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ForwardingMode {
    Modern,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AssetKind {
    Server,
    Plugin,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkRuntime {
    LocalProcess,
    Kubernetes,
}

impl NetworkConfig {
    pub fn java_entry(&self) -> super::JavaEntry {
        let listener = self
            .instances
            .iter()
            .find(|item| item.kind == InstanceKind::Velocity)
            .and_then(|item| self.listener(&item.listener))
            .or_else(|| {
                self.listeners
                    .iter()
                    .find(|item| item.protocol == ListenerProtocol::JavaTcp)
            });
        listener
            .map(|item| super::JavaEntry {
                bind_host: item.bind_host.clone(),
                port: item.port,
                public_hosts: item.public_hosts.clone(),
                preferred_public_host: item.public_hosts.first().cloned(),
            })
            .unwrap_or_default()
    }

    pub fn bedrock_entry(&self) -> super::BedrockEntry {
        self.listeners
            .iter()
            .find(|item| item.protocol == ListenerProtocol::BedrockUdp)
            .map(|item| super::BedrockEntry {
                mode: super::BedrockMode::Enabled,
                host: item.bind_host.clone(),
                port: item.port,
            })
            .unwrap_or_else(|| super::BedrockEntry {
                mode: super::BedrockMode::Disabled,
                host: "127.0.0.1".to_string(),
                port: 19132,
            })
    }

    pub fn fallback_server(&self) -> &str {
        &self.routes[0].target
    }
    pub fn forwarding_secret_file(&self) -> &str {
        &self.forwarding.secret_file
    }
    pub fn online_mode(&self) -> bool {
        self.auth.online_mode
    }
    pub fn listener(&self, id: &str) -> Option<&NetworkListener> {
        self.listeners.iter().find(|item| item.id == id)
    }
}
