use serde::{Deserialize, Serialize};

use crate::id::InstanceId;
use crate::instance::InstanceKind;

use super::plugin::PluginId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootstrapFacts {
    pub database_available: bool,
    pub daemon_http_available: bool,
    pub installed_binaries: InstalledBinaries,
    pub existing_instances: Vec<InstanceSummary>,
    pub assets: Vec<AssetSummary>,
    pub plugin_downloads: Vec<PluginId>,
    pub ports: PortFacts,
    pub filesystem: FilesystemFacts,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct InstalledBinaries {
    pub daemon: bool,
    pub cli: bool,
    pub java: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceSummary {
    pub id: InstanceId,
    pub kind: InstanceKind,
    pub managed: bool,
    pub server_port: u16,
    pub running: bool,
    pub config_stale: bool,
    pub plugins_changed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServerProject {
    Paper,
    Folia,
    Velocity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetRef {
    Server(ServerProject),
    Plugin(PluginId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetSummary {
    pub asset: AssetRef,
    pub verified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortFacts {
    pub tcp_in_use: Vec<u16>,
    pub udp_in_use: Vec<u16>,
    pub backend_range_start: u16,
    pub backend_range_end: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilesystemFacts {
    pub daemon_http_token_exists: bool,
    pub forwarding_secret_exists: bool,
    pub proxy_dir: DirectoryState,
    pub hub_dir: DirectoryState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DirectoryState {
    Absent,
    Managed,
    Unmanaged,
}

impl BootstrapFacts {
    pub fn find_instance(&self, id: &str) -> Option<&InstanceSummary> {
        self.existing_instances
            .iter()
            .find(|instance| instance.id.as_str() == id)
    }

    pub fn has_server_asset(&self, project: ServerProject) -> bool {
        self.assets
            .iter()
            .any(|asset| asset.asset == AssetRef::Server(project) && asset.verified)
    }

    pub fn has_plugin_asset(&self, plugin: PluginId) -> bool {
        self.assets
            .iter()
            .any(|asset| asset.asset == AssetRef::Plugin(plugin) && asset.verified)
    }

    pub fn can_fetch_plugin(&self, plugin: PluginId) -> bool {
        self.plugin_downloads.contains(&plugin)
    }
}

impl AssetSummary {
    pub fn server(project: ServerProject) -> Self {
        Self {
            asset: AssetRef::Server(project),
            verified: true,
        }
    }

    pub fn plugin(plugin: PluginId) -> Self {
        Self {
            asset: AssetRef::Plugin(plugin),
            verified: true,
        }
    }
}
