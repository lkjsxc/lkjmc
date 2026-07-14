mod plugins;

use super::desired::{DesiredInstance, DesiredNetwork};
use super::facts::{BootstrapFacts, ServerProject};
use super::plugin::PluginId;
use crate::id::InstanceId;
use crate::instance::InstanceKind;
use plugins::install_plugins;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BootstrapEffect {
    EnsureRoots,
    EnsureMigrations,
    GenerateDaemonHttpToken {
        path: String,
    },
    GenerateForwardingSecret {
        path: String,
    },
    SyncServerAsset {
        project: ServerProject,
    },
    RegisterLocalPlugin {
        plugin: PluginId,
    },
    SyncPluginAsset {
        plugin: PluginId,
    },
    ReconcileInstance {
        id: InstanceId,
        kind: InstanceKind,
        server_port: u16,
        memory_mb: u32,
        bind_host: String,
        public_hosts: Vec<String>,
        backend_address: Option<String>,
        forwarding_secret_file: String,
        online_mode: bool,
        daemon_http_url: String,
        daemon_http_token_file: String,
    },
    RenderInstance {
        id: InstanceId,
    },
    InstallPlugin {
        id: InstanceId,
        plugin: PluginId,
    },
    StartInstance {
        id: InstanceId,
    },
    StopInstance {
        id: InstanceId,
    },
    RestartInstance {
        id: InstanceId,
    },
    WaitForReadiness {
        id: InstanceId,
    },
}
pub fn sync_server_if_missing(
    project: ServerProject,
    facts: &BootstrapFacts,
    effects: &mut Vec<BootstrapEffect>,
) {
    if !facts.has_server_asset(project) {
        effects.push(BootstrapEffect::SyncServerAsset { project });
    }
}
pub fn register_local_if_missing(
    plugin: PluginId,
    facts: &BootstrapFacts,
    effects: &mut Vec<BootstrapEffect>,
) {
    if !facts.has_plugin_asset(plugin) {
        effects.push(BootstrapEffect::RegisterLocalPlugin { plugin });
    }
}
pub fn sync_plugin_if_missing(
    plugin: PluginId,
    facts: &BootstrapFacts,
    effects: &mut Vec<BootstrapEffect>,
) {
    if !facts.has_plugin_asset(plugin) {
        effects.push(BootstrapEffect::SyncPluginAsset { plugin });
    }
}
pub fn add_instance_effects(
    desired: &DesiredNetwork,
    facts: &BootstrapFacts,
    effects: &mut Vec<BootstrapEffect>,
    optional_plugins: &[PluginId],
) {
    let hub_plugins = optional_plugins
        .iter()
        .copied()
        .filter(|plugin| matches!(plugin, PluginId::ViaVersion | PluginId::ViaBackwards))
        .collect::<Vec<_>>();
    let proxy_plugins = optional_plugins
        .iter()
        .copied()
        .filter(|plugin| matches!(plugin, PluginId::Geyser | PluginId::Floodgate))
        .collect::<Vec<_>>();
    add_one_instance(
        &desired.backends[0],
        facts,
        effects,
        PluginId::LkjmcPaper,
        &hub_plugins,
        desired,
        None,
    );
    let backend_address = format!("127.0.0.1:{}", desired.backends[0].server_port);
    add_one_instance(
        &desired.proxy,
        facts,
        effects,
        PluginId::LkjmcVelocity,
        &proxy_plugins,
        desired,
        Some(backend_address),
    );
}
fn add_one_instance(
    desired: &DesiredInstance,
    facts: &BootstrapFacts,
    effects: &mut Vec<BootstrapEffect>,
    required_plugin: PluginId,
    optional_plugins: &[PluginId],
    network: &DesiredNetwork,
    backend_address: Option<String>,
) {
    let existing = facts.find_instance(desired.id.as_str());
    let needs_reconcile = match existing {
        Some(instance) => {
            !instance.managed
                || instance.kind != desired.kind
                || instance.server_port != desired.server_port
        }
        None => true,
    };
    if needs_reconcile {
        effects.push(BootstrapEffect::ReconcileInstance {
            id: desired.id.clone(),
            kind: desired.kind,
            server_port: desired.server_port,
            memory_mb: desired.memory_mb,
            bind_host: desired.bind_host.clone(),
            public_hosts: desired.public_hosts.clone(),
            backend_address,
            forwarding_secret_file: network.forwarding.secret_file.clone(),
            online_mode: network.forwarding.online_mode,
            daemon_http_url: network.daemon_http.address.clone(),
            daemon_http_token_file: network.daemon_http.token_file.clone(),
        });
        effects.push(BootstrapEffect::RenderInstance {
            id: desired.id.clone(),
        });
        install_plugins(effects, desired, required_plugin, optional_plugins);
        effects.push(BootstrapEffect::StartInstance {
            id: desired.id.clone(),
        });
        effects.push(BootstrapEffect::WaitForReadiness {
            id: desired.id.clone(),
        });
        return;
    }
    if existing.is_some_and(|instance| instance.config_stale || instance.plugins_changed) {
        effects.push(BootstrapEffect::RenderInstance {
            id: desired.id.clone(),
        });
        install_plugins(effects, desired, required_plugin, optional_plugins);
        effects.push(BootstrapEffect::RestartInstance {
            id: desired.id.clone(),
        });
        effects.push(BootstrapEffect::WaitForReadiness {
            id: desired.id.clone(),
        });
    } else if existing.is_some_and(|instance| !instance.running) {
        effects.push(BootstrapEffect::StartInstance {
            id: desired.id.clone(),
        });
        effects.push(BootstrapEffect::WaitForReadiness {
            id: desired.id.clone(),
        });
    }
}
