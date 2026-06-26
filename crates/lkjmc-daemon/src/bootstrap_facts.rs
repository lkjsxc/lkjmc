use std::path::Path;

use lkjmc_core::bootstrap::{
    AssetRef, AssetSummary, BootstrapFacts, DirectoryState, FilesystemFacts, InstalledBinaries,
    InstanceSummary, PluginId, PortFacts, ServerProject,
};
use lkjmc_core::id::InstanceId;
use lkjmc_core::instance::InstanceKind;
use postgres::Client;

use crate::app::AppState;

pub fn gather(state: &AppState) -> BootstrapFacts {
    let Some(database_url) = state.database_url() else {
        return without_database(state);
    };
    let mut client = match lkjmc_store::pool::connect(&database_url) {
        Ok(client) => client,
        Err(_) => return without_database(state),
    };
    let existing_instances = instances(&mut client);
    BootstrapFacts {
        database_available: true,
        daemon_http_available: state.http_listener().is_some(),
        installed_binaries: InstalledBinaries {
            daemon: true,
            cli: true,
            java: true,
        },
        filesystem: filesystem(state, &existing_instances),
        assets: assets(&mut client),
        plugin_downloads: crate::plugin_downloads::supported(),
        ports: ports(&mut client),
        existing_instances,
    }
}

fn without_database(state: &AppState) -> BootstrapFacts {
    BootstrapFacts {
        database_available: false,
        daemon_http_available: state.http_listener().is_some(),
        installed_binaries: InstalledBinaries::default(),
        existing_instances: Vec::new(),
        assets: Vec::new(),
        plugin_downloads: Vec::new(),
        ports: PortFacts {
            tcp_in_use: Vec::new(),
            udp_in_use: Vec::new(),
            backend_range_start: 25566,
            backend_range_end: 25665,
        },
        filesystem: filesystem(state, &[]),
    }
}

fn instances(client: &mut Client) -> Vec<InstanceSummary> {
    lkjmc_store::instance::list(client)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| {
            let id = InstanceId::parse(row.id.clone()).ok()?;
            let kind = kind(&row.kind)?;
            let config = lkjmc_store::instance::config(client, &row.id)
                .ok()
                .flatten();
            Some(InstanceSummary {
                id,
                kind,
                managed: true,
                server_port: config
                    .and_then(|value| value.get("serverPort").and_then(|port| port.as_u64()))
                    .and_then(|port| u16::try_from(port).ok())
                    .unwrap_or(0),
                running: row.healthy.unwrap_or(false),
                config_stale: false,
                plugins_changed: false,
            })
        })
        .collect()
}

fn assets(client: &mut Client) -> Vec<AssetSummary> {
    let mut values = lkjmc_store::asset::list(client)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|asset| asset_ref(&asset.asset_kind, &asset.project, &asset.platform))
        .map(|asset| AssetSummary {
            asset,
            verified: true,
        })
        .collect::<Vec<_>>();
    for jar in lkjmc_store::jar::list(client).unwrap_or_default() {
        if let Some(project) = server_project(&jar.project) {
            values.push(AssetSummary::server(project));
        }
    }
    values
}

fn ports(client: &mut Client) -> PortFacts {
    let rows = client
        .query("select port from instance_ports order by port", &[])
        .unwrap_or_default();
    PortFacts {
        tcp_in_use: rows
            .into_iter()
            .filter_map(|row| u16::try_from(row.get::<_, i32>(0)).ok())
            .collect(),
        udp_in_use: Vec::new(),
        backend_range_start: 25566,
        backend_range_end: 25665,
    }
}

fn filesystem(state: &AppState, instances: &[InstanceSummary]) -> FilesystemFacts {
    FilesystemFacts {
        daemon_http_token_exists: Path::new("/etc/lkjmc/daemon-http.token").exists(),
        forwarding_secret_exists: Path::new("/etc/lkjmc/forwarding.secret").exists(),
        proxy_dir: dir_state(state, "proxy", instances),
        hub_dir: dir_state(state, "hub", instances),
    }
}

fn dir_state(state: &AppState, id: &str, instances: &[InstanceSummary]) -> DirectoryState {
    if instances.iter().any(|instance| instance.id.as_str() == id) {
        return DirectoryState::Managed;
    }
    if Path::new(&state.data_root()).join(id).exists() {
        DirectoryState::Unmanaged
    } else {
        DirectoryState::Absent
    }
}

fn asset_ref(kind: &str, project: &str, platform: &str) -> Option<AssetRef> {
    match kind {
        "server" => server_project(project).map(AssetRef::Server),
        "plugin" => plugin_id(project)
            .or_else(|| plugin_id(platform))
            .map(AssetRef::Plugin),
        _ => None,
    }
}

fn server_project(project: &str) -> Option<ServerProject> {
    match project {
        "paper" => Some(ServerProject::Paper),
        "folia" => Some(ServerProject::Folia),
        "velocity" => Some(ServerProject::Velocity),
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

fn kind(value: &str) -> Option<InstanceKind> {
    match value {
        "velocity" => Some(InstanceKind::Velocity),
        "paper" => Some(InstanceKind::Paper),
        "folia" => Some(InstanceKind::Folia),
        "vanilla-custom" => Some(InstanceKind::VanillaCustom),
        "modded-custom" => Some(InstanceKind::ModdedCustom),
        _ => None,
    }
}
