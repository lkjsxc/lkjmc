use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::time::Duration;

use lkjmc_core::instance::{DesiredState, InstanceKind};
use lkjmc_core::network_intent::{
    self, InspectionOutcome, NetworkInspection, NetworkObservation, ResourceObservation,
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::app::AppState;
use crate::runtime::RuntimeGoal;

pub(super) fn inspect(state: &AppState) -> Result<NetworkInspection, String> {
    let config = state
        .runtime_config()?
        .ok_or("runtime config is unavailable")?;
    let observed = observation(state, &config.network)?;
    let mut inspection = network_intent::inspect(&config.network, &observed);
    if inspection.outcome != InspectionOutcome::Blocked {
        inspection
            .unsupported
            .extend(asset_failures(&config.network));
        if !inspection.unsupported.is_empty() {
            inspection.outcome = InspectionOutcome::Blocked;
            inspection.changes.clear();
        }
    }
    Ok(inspection)
}

fn observation(
    state: &AppState,
    intent: &lkjmc_core::config::NetworkConfig,
) -> Result<NetworkObservation, String> {
    if state.database_url().is_none() {
        return Ok(NetworkObservation::default());
    }
    let (intent_digest, stored) = {
        let mut client = state.database_connection()?;
        let desired = lkjmc_store::network_intent::latest_desired(&mut client)
            .map_err(|error| error.to_string())?;
        let digest = desired
            .filter(|value| value.intent_digest == intent.digest())
            .map(|value| value.intent_digest);
        let mut rows = BTreeMap::new();
        for instance in &intent.instances {
            let row = lkjmc_store::instance::get(&mut client, &instance.id)
                .map_err(|error| error.to_string())?;
            let config = lkjmc_store::instance::config(&mut client, &instance.id)
                .map_err(|error| error.to_string())?;
            rows.insert(instance.id.clone(), (row, config));
        }
        (digest, rows)
    };
    let mut resources = BTreeMap::new();
    for instance in &intent.instances {
        let has_shape = stored
            .get(&instance.id)
            .is_some_and(|(row, config)| row.is_some() && config.is_some());
        let runtime = if has_shape {
            crate::runtime::reconcile::reconcile(
                state,
                &instance.id,
                RuntimeGoal::Observe,
                Uuid::new_v4(),
            )?
        } else {
            state
                .runtime()
                .runtime_status(&instance.id)?
                .unwrap_or_else(|| crate::runtime::RuntimeObservation::absent("runtime is absent"))
        };
        let listener = intent
            .listener(&instance.listener)
            .ok_or_else(|| format!("listener missing for {}", instance.id))?;
        let address = socket_address(&listener.bind_host, listener.port)?;
        let files_ready = rendered_files_ready(state, &instance.id, instance.kind, listener.port);
        let listener_ready = runtime.healthy
            && TcpStream::connect_timeout(&address, Duration::from_millis(50)).is_ok();
        let blocked = runtime_block(&runtime, address, instance.desired_state);
        resources.insert(
            instance.id.clone(),
            ResourceObservation {
                spec_digest: if files_ready && has_shape {
                    intent.resource_digest(&instance.id)
                } else {
                    "drift".to_string()
                },
                ready: runtime.healthy && listener_ready,
                blocked,
            },
        );
    }
    Ok(NetworkObservation {
        intent_digest,
        forwarding_secret_ready: private_file(&intent.forwarding.secret_file),
        resources,
    })
}

fn runtime_block(
    runtime: &crate::runtime::RuntimeObservation,
    address: SocketAddr,
    desired: DesiredState,
) -> Option<String> {
    if !runtime.healthy && !runtime.observed_state.contains("absent") {
        return Some("runtime identity is stale or unowned; apply denied".to_string());
    }
    if desired == DesiredState::Running && !runtime.healthy && TcpListener::bind(address).is_err() {
        return Some(format!(
            "unowned process or listener occupies {address}; apply denied"
        ));
    }
    None
}

fn rendered_files_ready(state: &AppState, id: &str, kind: InstanceKind, port: u16) -> bool {
    let root = Path::new(&state.data_root()).join(id);
    let files: &[(&str, String)] = match kind {
        InstanceKind::Velocity => &[
            ("velocity.toml", format!("{port}")),
            ("forwarding.secret", String::new()),
        ],
        _ => &[
            ("server.properties", format!("server-port={port}")),
            ("eula.txt", "eula=true".to_string()),
            ("spigot.yml", String::new()),
            ("config/paper-global.yml", "velocity:".to_string()),
        ],
    };
    files.iter().all(|(relative, needle)| {
        let path = root.join(relative);
        private_file(path.to_string_lossy().as_ref())
            && std::fs::read_to_string(path).is_ok_and(|text| text.contains(needle))
    })
}

fn socket_address(host: &str, port: u16) -> Result<SocketAddr, String> {
    format!("{host}:{port}")
        .parse()
        .map_err(|error| format!("invalid listener address: {error}"))
}

fn asset_failures(intent: &lkjmc_core::config::NetworkConfig) -> Vec<String> {
    intent
        .assets
        .iter()
        .filter(|asset| asset.required)
        .filter_map(|asset| {
            if asset.kind == lkjmc_core::config::AssetKind::Plugin {
                return Some(format!(
                    "network capability unsupported: plugin asset adoption: {}",
                    asset.id
                ));
            }
            match file_sha256(&asset.path) {
                Ok(actual) if actual.eq_ignore_ascii_case(&asset.sha256) => None,
                Ok(_) => Some(format!("required asset digest mismatch: {}", asset.id)),
                Err(error) => Some(format!("required asset unavailable: {}: {error}", asset.id)),
            }
        })
        .collect()
}

fn file_sha256(path: &str) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn private_file(path: &str) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.is_file() && metadata.permissions().mode() & 0o077 == 0
    }
    #[cfg(not(unix))]
    metadata.is_file()
}
