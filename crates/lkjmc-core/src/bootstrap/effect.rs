use serde::{Deserialize, Serialize};

use crate::id::InstanceId;
use crate::instance::InstanceKind;

use super::desired::{DesiredInstance, DesiredNetwork};
use super::facts::{BootstrapFacts, ServerProject};
use super::plugin::PluginId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BootstrapEffect {
    EnsureRoots,
    EnsureMigrations,
    GenerateDaemonHttpToken,
    GenerateForwardingSecret,
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
) {
    let hub = &desired.backends[0];
    add_one_instance(hub, facts, effects, PluginId::LkjmcPaper);
    add_one_instance(&desired.proxy, facts, effects, PluginId::LkjmcVelocity);
}

fn add_one_instance(
    desired: &DesiredInstance,
    facts: &BootstrapFacts,
    effects: &mut Vec<BootstrapEffect>,
    required_plugin: PluginId,
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
        });
        effects.push(BootstrapEffect::RenderInstance {
            id: desired.id.clone(),
        });
        effects.push(BootstrapEffect::InstallPlugin {
            id: desired.id.clone(),
            plugin: required_plugin,
        });
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
        effects.push(BootstrapEffect::InstallPlugin {
            id: desired.id.clone(),
            plugin: required_plugin,
        });
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
