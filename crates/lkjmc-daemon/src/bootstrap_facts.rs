mod catalog;

use std::path::Path;

use catalog::{asset_ref, kind, server_project};
use lkjmc_core::bootstrap::{
    AssetSummary, BootstrapFacts, DirectoryState, FilesystemFacts, InstalledBinaries,
    InstanceSummary, PortFacts,
};
use lkjmc_core::id::InstanceId;
use postgres::Client;

use crate::app::AppState;

pub fn gather(state: &AppState) -> BootstrapFacts {
    if state.database_url().is_none() {
        return without_database(state);
    }
    let mut client = match state.database_connection() {
        Ok(client) => client,
        Err(_) => return without_database(state),
    };
    let existing_instances = instances(&mut client);
    BootstrapFacts {
        database_available: true,
        schema_current: schema_current(&mut client),
        daemon_http_available: state.http_listener().is_some(),
        installed_binaries: InstalledBinaries {
            daemon: true,
            cli: true,
            java: true,
        },
        filesystem: filesystem(state, &existing_instances),
        assets: assets(&mut client),
        plugin_downloads: crate::plugin_downloads::supported(),
        ports: ports(state, &mut client),
        existing_instances,
    }
}

fn without_database(state: &AppState) -> BootstrapFacts {
    BootstrapFacts {
        database_available: false,
        schema_current: false,
        daemon_http_available: state.http_listener().is_some(),
        installed_binaries: InstalledBinaries::default(),
        existing_instances: Vec::new(),
        assets: Vec::new(),
        plugin_downloads: Vec::new(),
        ports: empty_ports(state),
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
                running: row.healthy.unwrap_or(false) && pid_alive(row.pid),
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

fn pid_alive(pid: Option<i32>) -> bool {
    pid.and_then(|value| u32::try_from(value).ok())
        .is_some_and(crate::process::group_exists)
}

fn ports(state: &AppState, client: &mut Client) -> PortFacts {
    let rows = client
        .query("select port from instance_ports order by port", &[])
        .unwrap_or_default();
    let mut facts = empty_ports(state);
    facts.tcp_in_use = rows
        .into_iter()
        .filter_map(|row| u16::try_from(row.get::<_, i32>(0)).ok())
        .collect();
    facts
}

fn empty_ports(state: &AppState) -> PortFacts {
    let config = state.runtime_config().ok().flatten();
    PortFacts {
        tcp_in_use: Vec::new(),
        udp_in_use: Vec::new(),
        backend_range_start: config
            .as_ref()
            .map(|config| config.runtime.port_range_start)
            .unwrap_or(25566),
        backend_range_end: config
            .as_ref()
            .map(|config| config.runtime.port_range_end)
            .unwrap_or(25665),
    }
}

fn filesystem(state: &AppState, instances: &[InstanceSummary]) -> FilesystemFacts {
    let config = state.runtime_config().ok().flatten();
    let token = config
        .as_ref()
        .map(|config| config.daemon_http.token_file.as_str())
        .unwrap_or("/etc/lkjmc/daemon-http.token");
    let forwarding = config
        .as_ref()
        .map(|config| config.network.forwarding_secret_file.as_str())
        .unwrap_or("/etc/lkjmc/forwarding.secret");
    FilesystemFacts {
        roots_ready: roots_ready(state),
        daemon_http_token_exists: Path::new(token).exists(),
        forwarding_secret_exists: Path::new(forwarding).exists(),
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

fn roots_ready(state: &AppState) -> bool {
    let roots = [
        state.config_root(),
        state.data_root(),
        state.log_root(),
        state.jar_root(),
        state.asset_root(),
    ];
    let roots_ready = roots
        .iter()
        .all(|root| !root.is_empty() && Path::new(root).is_dir());
    let socket_ready = Path::new(&state.socket_path())
        .parent()
        .is_some_and(Path::is_dir);
    roots_ready && socket_ready
}

fn schema_current(client: &mut Client) -> bool {
    let Ok(row) = client.query_one("select to_regclass('public.schema_migrations')::text", &[])
    else {
        return false;
    };
    let table: Option<String> = row.get(0);
    if table.is_none() {
        return false;
    }
    let Ok(rows) = client.query("select version from schema_migrations", &[]) else {
        return false;
    };
    let applied = rows
        .into_iter()
        .map(|row| row.get::<_, i32>(0))
        .collect::<Vec<_>>();
    lkjmc_store::migrate::migrations()
        .into_iter()
        .all(|migration| applied.contains(&migration.version))
}
