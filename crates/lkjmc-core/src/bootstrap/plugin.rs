use serde::{Deserialize, Serialize};

use crate::config::{BedrockMode, PluginMode};

use super::diagnostic::{BootstrapDiagnostic, DiagnosticCode};
use super::effect::{sync_plugin_if_missing, BootstrapEffect};
use super::facts::BootstrapFacts;
use super::BootstrapRequest;

pub type PluginPolicy = crate::config::PluginsConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginId {
    LkjmcPaper,
    LkjmcVelocity,
    #[serde(rename = "viaversion")]
    ViaVersion,
    #[serde(rename = "viabackwards")]
    ViaBackwards,
    Geyser,
    Floodgate,
}

impl PluginId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LkjmcPaper => "lkjmc-paper",
            Self::LkjmcVelocity => "lkjmc-velocity",
            Self::ViaVersion => "viaversion",
            Self::ViaBackwards => "viabackwards",
            Self::Geyser => "geyser",
            Self::Floodgate => "floodgate",
        }
    }
}

pub fn add_via_effects(
    request: &BootstrapRequest,
    facts: &BootstrapFacts,
    effects: &mut Vec<BootstrapEffect>,
    diagnostics: &mut Vec<BootstrapDiagnostic>,
) -> Vec<PluginId> {
    if request.plugin_policy.viaversion.mode == PluginMode::Disabled
        && request.plugin_policy.viabackwards.mode == PluginMode::Disabled
    {
        return Vec::new();
    }
    if !plugin_usable(PluginId::ViaVersion, facts) {
        diagnostics.push(BootstrapDiagnostic::warning(
            DiagnosticCode::ViaWithdrawn,
            "ViaVersion is not available as a hash-verified plugin asset",
        ));
        if request.plugin_policy.viabackwards.mode != PluginMode::Disabled {
            diagnostics.push(BootstrapDiagnostic::warning(
                DiagnosticCode::ViaBackwardsDependency,
                "ViaBackwards was withdrawn because ViaVersion is unavailable",
            ));
        }
        return Vec::new();
    }
    sync_plugin_if_missing(PluginId::ViaVersion, facts, effects);
    let mut plugins = vec![PluginId::ViaVersion];
    if request.plugin_policy.viabackwards.mode == PluginMode::Disabled {
        return plugins;
    }
    if plugin_usable(PluginId::ViaBackwards, facts) {
        sync_plugin_if_missing(PluginId::ViaBackwards, facts, effects);
        plugins.push(PluginId::ViaBackwards);
    } else {
        diagnostics.push(BootstrapDiagnostic::warning(
            DiagnosticCode::ViaBackwardsDependency,
            "ViaBackwards is not available as a hash-verified plugin asset",
        ));
    }
    plugins
}

pub fn add_bedrock_effects(
    request: &BootstrapRequest,
    facts: &BootstrapFacts,
    effects: &mut Vec<BootstrapEffect>,
    diagnostics: &mut Vec<BootstrapDiagnostic>,
) -> Vec<PluginId> {
    if request.bedrock_entry.mode == BedrockMode::Disabled {
        return Vec::new();
    }
    if facts.ports.udp_in_use.contains(&request.bedrock_entry.port) {
        add_bedrock_unavailable(request, diagnostics, "Bedrock UDP port is unavailable");
        return Vec::new();
    }
    if plugin_usable(PluginId::Geyser, facts) && plugin_usable(PluginId::Floodgate, facts) {
        sync_plugin_if_missing(PluginId::Geyser, facts, effects);
        sync_plugin_if_missing(PluginId::Floodgate, facts, effects);
        return vec![PluginId::Geyser, PluginId::Floodgate];
    }
    add_bedrock_unavailable(
        request,
        diagnostics,
        "Geyser or Floodgate is not available as a hash-verified plugin asset",
    );
    Vec::new()
}

fn add_bedrock_unavailable(
    request: &BootstrapRequest,
    diagnostics: &mut Vec<BootstrapDiagnostic>,
    message: &'static str,
) {
    if request.bedrock_entry.mode == BedrockMode::Enabled {
        diagnostics.push(BootstrapDiagnostic::blocking(
            DiagnosticCode::BedrockBlocked,
            message,
        ));
    } else {
        diagnostics.push(BootstrapDiagnostic::warning(
            DiagnosticCode::BedrockWithdrawn,
            message,
        ));
    }
}

fn plugin_usable(plugin: PluginId, facts: &BootstrapFacts) -> bool {
    facts.has_plugin_asset(plugin) || facts.can_fetch_plugin(plugin)
}
