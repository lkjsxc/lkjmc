use super::desired::DesiredNetwork;
use super::diagnostic::{BootstrapDiagnostic, DiagnosticCode, DiagnosticSeverity};
use super::effect::{
    add_instance_effects, register_local_if_missing, sync_server_if_missing, BootstrapEffect,
};
use super::facts::{BootstrapFacts, DirectoryState, ServerProject};
use super::plan::BootstrapPlan;
use super::plugin::{add_bedrock_effects, add_via_effects, PluginId};
use super::ports::allocate_backend_port;
use super::BootstrapRequest;

pub fn plan_bootstrap(request: &BootstrapRequest, facts: &BootstrapFacts) -> BootstrapPlan {
    let mut diagnostics = Vec::new();
    add_blockers(request, facts, &mut diagnostics);
    let hub_port = choose_hub_port(facts, &mut diagnostics);
    let desired = DesiredNetwork::playable(&request.java_entry, hub_port.unwrap_or(25566));
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Blocking)
    {
        return BootstrapPlan::new(desired, Vec::new(), diagnostics, request.dry_run);
    }
    let mut effects = Vec::new();
    add_secret_effects(facts, &mut effects);
    let optional_plugins = add_asset_effects(request, facts, &mut effects, &mut diagnostics);
    add_instance_effects(&desired, facts, &mut effects, &optional_plugins);
    BootstrapPlan::new(desired, effects, diagnostics, request.dry_run)
}

fn add_blockers(
    request: &BootstrapRequest,
    facts: &BootstrapFacts,
    diagnostics: &mut Vec<BootstrapDiagnostic>,
) {
    if !request.accept_minecraft_eula {
        diagnostics.push(BootstrapDiagnostic::blocking(
            DiagnosticCode::MinecraftEulaRequired,
            "pass --accept-minecraft-eula or set LKJMC_ACCEPT_MINECRAFT_EULA=1",
        ));
    }
    if !facts.database_available {
        diagnostics.push(BootstrapDiagnostic::blocking(
            DiagnosticCode::DatabaseUnavailable,
            "PostgreSQL is required for playable bootstrap",
        ));
    }
    block_unmanaged(facts.filesystem.proxy_dir, "proxy", diagnostics);
    block_unmanaged(facts.filesystem.hub_dir, "hub", diagnostics);
    if java_port_conflicts(request.java_entry.port, facts) {
        diagnostics.push(BootstrapDiagnostic::blocking(
            DiagnosticCode::JavaPortUnavailable,
            "Java proxy TCP port is already in use",
        ));
    }
}

fn choose_hub_port(
    facts: &BootstrapFacts,
    diagnostics: &mut Vec<BootstrapDiagnostic>,
) -> Option<u16> {
    if let Some(instance) = facts.find_instance("hub") {
        return Some(instance.server_port);
    }
    let port = allocate_backend_port(25566, &facts.ports);
    if let Some(chosen) = port {
        if chosen != 25566 {
            diagnostics.push(BootstrapDiagnostic::info(
                DiagnosticCode::PortReallocated,
                format!("backend port 25566 is busy; using {chosen}"),
            ));
        }
    } else {
        diagnostics.push(BootstrapDiagnostic::blocking(
            DiagnosticCode::BackendPortUnavailable,
            "no backend TCP port is available in the configured range",
        ));
    }
    port
}

fn add_secret_effects(facts: &BootstrapFacts, effects: &mut Vec<BootstrapEffect>) {
    if !facts.filesystem.daemon_http_token_exists {
        effects.push(BootstrapEffect::GenerateDaemonHttpToken);
    }
    if !facts.filesystem.forwarding_secret_exists {
        effects.push(BootstrapEffect::GenerateForwardingSecret);
    }
}

fn add_asset_effects(
    request: &BootstrapRequest,
    facts: &BootstrapFacts,
    effects: &mut Vec<BootstrapEffect>,
    diagnostics: &mut Vec<BootstrapDiagnostic>,
) -> Vec<PluginId> {
    sync_server_if_missing(ServerProject::Velocity, facts, effects);
    sync_server_if_missing(ServerProject::Paper, facts, effects);
    register_local_if_missing(PluginId::LkjmcVelocity, facts, effects);
    register_local_if_missing(PluginId::LkjmcPaper, facts, effects);
    let mut plugins = add_via_effects(request, facts, effects, diagnostics);
    plugins.extend(add_bedrock_effects(request, facts, effects, diagnostics));
    plugins
}

fn block_unmanaged(
    state: DirectoryState,
    id: &'static str,
    diagnostics: &mut Vec<BootstrapDiagnostic>,
) {
    if state == DirectoryState::Unmanaged {
        diagnostics.push(BootstrapDiagnostic::blocking(
            DiagnosticCode::UnmanagedDirectoryConflict,
            format!("{id} instance path exists but is not managed by lkjmc"),
        ));
    }
}

fn java_port_conflicts(port: u16, facts: &BootstrapFacts) -> bool {
    let owned_by_proxy = facts
        .find_instance("proxy")
        .is_some_and(|instance| instance.managed && instance.server_port == port);
    facts.ports.tcp_in_use.contains(&port) && !owned_by_proxy
}
