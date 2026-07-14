use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;

use lkjmc_core::network_intent::{
    self, InspectionOutcome, NetworkInspection, NetworkObservation, ResourceObservation,
};
use sha2::{Digest, Sha256};

use crate::app::AppState;

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

fn observation(
    state: &AppState,
    intent: &lkjmc_core::config::NetworkConfig,
) -> Result<NetworkObservation, String> {
    if state.database_url().is_none() {
        return Ok(NetworkObservation::default());
    }
    let mut client = state.database_connection()?;
    let desired = match lkjmc_store::network_intent::latest_desired(&mut client) {
        Ok(value) => value,
        Err(_) => return Ok(NetworkObservation::default()),
    };
    let Some(desired) = desired else {
        return Ok(NetworkObservation::default());
    };
    if desired.intent_digest != intent.digest() {
        return Ok(NetworkObservation::default());
    }
    let mut resources = BTreeMap::new();
    for instance in &intent.instances {
        let Some(row) = lkjmc_store::instance::get(&mut client, &instance.id)
            .map_err(|error| error.to_string())?
        else {
            continue;
        };
        resources.insert(
            instance.id.clone(),
            ResourceObservation {
                spec_digest: intent.resource_digest(&instance.id),
                ready: row.healthy.unwrap_or(false),
            },
        );
    }
    Ok(NetworkObservation {
        intent_digest: Some(desired.intent_digest),
        forwarding_secret_ready: private_file(&intent.forwarding.secret_file),
        resources,
    })
}
