use crate::config::{BedrockEntry, BedrockMode, JavaEntry, PluginMode, PluginsConfig};
use crate::id::InstanceId;
use crate::instance::InstanceKind;

use super::*;

fn base_request(accept_minecraft_eula: bool) -> BootstrapRequest {
    BootstrapRequest {
        profile: BootstrapProfile::Playable,
        accept_minecraft_eula,
        java_entry: JavaEntry::default(),
        bedrock_entry: BedrockEntry::default(),
        plugin_policy: PluginsConfig::default(),
        runtime: BootstrapRuntimeSettings::default(),
        dry_run: false,
    }
}

fn base_facts() -> BootstrapFacts {
    BootstrapFacts {
        database_available: true,
        daemon_http_available: true,
        installed_binaries: InstalledBinaries {
            daemon: true,
            cli: true,
            java: true,
        },
        existing_instances: Vec::new(),
        assets: Vec::new(),
        plugin_downloads: vec![PluginId::ViaVersion, PluginId::ViaBackwards],
        ports: PortFacts {
            tcp_in_use: Vec::new(),
            udp_in_use: Vec::new(),
            backend_range_start: 25566,
            backend_range_end: 25665,
        },
        filesystem: FilesystemFacts {
            daemon_http_token_exists: true,
            forwarding_secret_exists: true,
            proxy_dir: DirectoryState::Absent,
            hub_dir: DirectoryState::Absent,
        },
    }
}

#[test]
fn eula_absence_blocks_playable_start() {
    let plan = plan_bootstrap(&base_request(false), &base_facts());
    assert_eq!(plan.outcome, PlannedOutcome::Blocked);
    assert!(plan.effects.is_empty());
    assert!(plan
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == DiagnosticCode::MinecraftEulaRequired));
}

#[test]
fn java_setup_continues_when_bedrock_auto_withdraws() {
    let mut facts = base_facts();
    facts.ports.udp_in_use.push(19132);
    let plan = plan_bootstrap(&base_request(true), &facts);
    assert_eq!(plan.outcome, PlannedOutcome::ReadyToApply);
    assert!(plan
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == DiagnosticCode::BedrockWithdrawn));
    assert!(plan.effects.iter().any(|effect| matches!(
        effect,
        BootstrapEffect::StartInstance { id } if id.as_str() == "proxy"
    )));
}

#[test]
fn viabackwards_withdraws_when_viaversion_unavailable() {
    let mut request = base_request(true);
    request.bedrock_entry.mode = BedrockMode::Disabled;
    let mut facts = base_facts();
    facts.plugin_downloads = vec![PluginId::ViaBackwards];
    let plan = plan_bootstrap(&request, &facts);
    assert!(plan
        .diagnostics
        .iter()
        .any(|diagnostic| { diagnostic.code == DiagnosticCode::ViaBackwardsDependency }));
    assert!(!plan.effects.iter().any(|effect| matches!(
        effect,
        BootstrapEffect::SyncPluginAsset {
            plugin: PluginId::ViaBackwards
        }
    )));
}

#[test]
fn backend_port_conflict_allocates_from_range() {
    let mut request = base_request(true);
    request.bedrock_entry.mode = BedrockMode::Disabled;
    let mut facts = base_facts();
    facts.ports.tcp_in_use.push(25566);
    facts.ports.backend_range_start = 25567;
    facts.ports.backend_range_end = 25567;
    let plan = plan_bootstrap(&request, &facts);
    assert_eq!(plan.desired_network.backends[0].server_port, 25567);
    assert!(plan.effects.iter().any(|effect| matches!(
        effect,
        BootstrapEffect::ReconcileInstance {
            id,
            server_port: 25567,
            ..
        } if id.as_str() == "hub"
    )));
}

#[test]
fn runtime_port_range_controls_hub_and_proxy_route() {
    let mut request = base_request(true);
    request.bedrock_entry.mode = BedrockMode::Disabled;
    let mut facts = base_facts();
    facts.ports.backend_range_start = 30000;
    facts.ports.backend_range_end = 30000;
    let plan = plan_bootstrap(&request, &facts);
    assert_eq!(plan.desired_network.backends[0].server_port, 30000);
    assert!(plan.effects.iter().any(|effect| matches!(
        effect,
        BootstrapEffect::ReconcileInstance {
            id,
            backend_address: Some(address),
            ..
        } if id.as_str() == "proxy" && address == "127.0.0.1:30000"
    )));
}

#[test]
fn converged_facts_plan_no_effects() {
    let mut request = base_request(true);
    request.bedrock_entry.mode = BedrockMode::Disabled;
    request.plugin_policy.viaversion.mode = PluginMode::Disabled;
    request.plugin_policy.viabackwards.mode = PluginMode::Disabled;
    let mut facts = base_facts();
    facts.assets = vec![
        AssetSummary::server(ServerProject::Velocity),
        AssetSummary::server(ServerProject::Paper),
        AssetSummary::plugin(PluginId::LkjmcVelocity),
        AssetSummary::plugin(PluginId::LkjmcPaper),
    ];
    facts.filesystem.proxy_dir = DirectoryState::Managed;
    facts.filesystem.hub_dir = DirectoryState::Managed;
    facts.existing_instances = vec![
        running("hub", InstanceKind::Paper, 25566),
        running("proxy", InstanceKind::Velocity, 25565),
    ];
    let plan = plan_bootstrap(&request, &facts);
    assert_eq!(plan.outcome, PlannedOutcome::AlreadyConverged);
    assert!(plan.effects.is_empty());
}

fn running(id: &'static str, kind: InstanceKind, server_port: u16) -> InstanceSummary {
    InstanceSummary {
        id: InstanceId::internal(id),
        kind,
        managed: true,
        server_port,
        running: true,
        config_stale: false,
        plugins_changed: false,
    }
}
