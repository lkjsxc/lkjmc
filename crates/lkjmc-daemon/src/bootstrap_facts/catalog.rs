use lkjmc_core::bootstrap::{AssetRef, PluginId, ServerProject};
use lkjmc_core::instance::InstanceKind;

pub fn asset_ref(kind: &str, project: &str, platform: &str) -> Option<AssetRef> {
    match kind {
        "server" => server_project(project).map(AssetRef::Server),
        "plugin" => plugin_id(project)
            .or_else(|| plugin_id(platform))
            .map(AssetRef::Plugin),
        _ => None,
    }
}

pub fn server_project(project: &str) -> Option<ServerProject> {
    match project {
        "paper" => Some(ServerProject::Paper),
        "folia" => Some(ServerProject::Folia),
        "velocity" => Some(ServerProject::Velocity),
        "purpur" => Some(ServerProject::Purpur),
        _ => None,
    }
}

pub fn kind(value: &str) -> Option<InstanceKind> {
    match value {
        "velocity" => Some(InstanceKind::Velocity),
        "paper" => Some(InstanceKind::Paper),
        "folia" => Some(InstanceKind::Folia),
        "purpur" => Some(InstanceKind::Purpur),
        "vanilla-custom" => Some(InstanceKind::VanillaCustom),
        "modded-custom" => Some(InstanceKind::ModdedCustom),
        _ => None,
    }
}

fn plugin_id(value: &str) -> Option<PluginId> {
    match value {
        "lkjmc-paper" => Some(PluginId::LkjmcPaper),
        "lkjmc-velocity" => Some(PluginId::LkjmcVelocity),
        "viaversion" => Some(PluginId::ViaVersion),
        "viabackwards" => Some(PluginId::ViaBackwards),
        "geyser" => Some(PluginId::Geyser),
        "floodgate" => Some(PluginId::Floodgate),
        _ => None,
    }
}
