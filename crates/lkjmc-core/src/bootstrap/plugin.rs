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
    let via_mode = request.plugin_policy.viaversion.mode;
    let backwards_mode = request.plugin_policy.viabackwards.mode;
    if via_mode == PluginMode::Disabled && backwards_mode == PluginMode::Disabled {
        return Vec::new();
    }
    if via_mode == PluginMode::Disabled {
        add_via_backwards_unavailable(
            request,
            diagnostics,
            "ViaBackwards requires ViaVersion, but ViaVersion is disabled",
        );
        return Vec::new();
    }
    if !plugin_usable(PluginId::ViaVersion, facts) {
        add_via_unavailable(
            request,
            diagnostics,
            "ViaVersion is not available as a hash-verified plugin asset",
        );
        return Vec::new();
    }
    sync_plugin_if_missing(PluginId::ViaVersion, facts, effects);
    let mut plugins = vec![PluginId::ViaVersion];
    if backwards_mode == PluginMode::Disabled {
        return plugins;
    }
    if plugin_usable(PluginId::ViaBackwards, facts) {
        sync_plugin_if_missing(PluginId::ViaBackwards, facts, effects);
        plugins.push(PluginId::ViaBackwards);
    } else {
        add_via_backwards_unavailable(
            request,
            diagnostics,
            "ViaBackwards is not available as a hash-verified plugin asset",
        );
    }
    plugins
}

fn add_via_unavailable(
    request: &BootstrapRequest,
    diagnostics: &mut Vec<BootstrapDiagnostic>,
    message: &'static str,
) {
    let required = request.plugin_policy.viaversion.mode == PluginMode::Enabled
        || request.plugin_policy.viabackwards.mode == PluginMode::Enabled;
    if required {
        diagnostics.push(BootstrapDiagnostic::blocking(
            DiagnosticCode::ViaWithdrawn,
            message,
        ));
    } else {
        diagnostics.push(BootstrapDiagnostic::warning(
            DiagnosticCode::ViaWithdrawn,
            message,
        ));
        if request.plugin_policy.viabackwards.mode != PluginMode::Disabled {
            diagnostics.push(BootstrapDiagnostic::warning(
                DiagnosticCode::ViaBackwardsDependency,
                "ViaBackwards was withdrawn because ViaVersion is unavailable",
            ));
        }
    }
}

fn add_via_backwards_unavailable(
    request: &BootstrapRequest,
    diagnostics: &mut Vec<BootstrapDiagnostic>,
    message: &'static str,
) {
    if request.plugin_policy.viabackwards.mode == PluginMode::Enabled {
        diagnostics.push(BootstrapDiagnostic::blocking(
            DiagnosticCode::ViaBackwardsDependency,
            message,
        ));
    } else {
        diagnostics.push(BootstrapDiagnostic::warning(
            DiagnosticCode::ViaBackwardsDependency,
            message,
        ));
    }
}

pub fn add_bedrock_effects(
    request: &BootstrapRequest,
    facts: &BootstrapFacts,
    effects: &mut Vec<BootstrapEffect>,
    diagnostics: &mut Vec<BootstrapDiagnostic>,
) -> Vec<PluginId> {
    if request.bedrock_entry.mode == BedrockMode::Disabled {
        if bedrock_required(request) {
            diagnostics.push(BootstrapDiagnostic::blocking(
                DiagnosticCode::BedrockBlocked,
                "Bedrock plugins are enabled but Bedrock entry is disabled",
            ));
        }
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
    if bedrock_required(request) {
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

fn bedrock_required(request: &BootstrapRequest) -> bool {
    request.bedrock_entry.mode == BedrockMode::Enabled
        || request.plugin_policy.geyser.mode == PluginMode::Enabled
        || request.plugin_policy.floodgate.mode == PluginMode::Enabled
}

fn plugin_usable(plugin: PluginId, facts: &BootstrapFacts) -> bool {
    facts.has_plugin_asset(plugin) || facts.can_fetch_plugin(plugin)
}
