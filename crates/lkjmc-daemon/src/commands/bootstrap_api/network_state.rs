use std::collections::BTreeMap;

use lkjmc_core::network_intent::{self, NetworkInspection, NetworkObservation, ResourceObservation};

use crate::app::AppState;

pub(super) fn inspect(state: &AppState) -> Result<NetworkInspection, String> {
    let config = state.runtime_config()?.ok_or("runtime config is unavailable")?;
    let observed = observation(state, &config.network)?;
    Ok(network_intent::inspect(&config.network, &observed))
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
    let Some(desired) = desired else { return Ok(NetworkObservation::default()); };
    if desired.intent_digest != intent.digest() {
        return Ok(NetworkObservation::default());
    }
    let mut resources = BTreeMap::new();
    for instance in &intent.instances {
        let Some(row) = lkjmc_store::instance::get(&mut client, &instance.id)
            .map_err(|error| error.to_string())? else { continue; };
        resources.insert(instance.id.clone(), ResourceObservation {
            spec_digest: intent.resource_digest(&instance.id),
            ready: row.healthy.unwrap_or(false),
        });
    }
    Ok(NetworkObservation { intent_digest: Some(desired.intent_digest), resources })
}
