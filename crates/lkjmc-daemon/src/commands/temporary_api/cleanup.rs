use std::fs;
use std::path::Path;

use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::commands::temporary_api::lifecycle::require_temp;
use crate::dispatch as api;
use crate::support::audit_helpers::audit;
use crate::support::instance_helpers::{
    body_string, runtime_running, stop_runtime, store, with_connection,
};

pub fn cleanup(
    state: &AppState,
    envelope: lkjmc_core::command::CommandEnvelope,
) -> lkjmc_core::command::CommandResponse {
    with_connection(state, envelope, |state, envelope, client| {
        let id = body_string(&envelope.body, "id")?;
        let force = envelope
            .body
            .get("force")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let temp = require_temp(client, &id)?;
        if runtime_running(state, &id)? {
            if !force {
                return Err("temporary instance is running; stop it or pass force".to_string());
            }
            stop_runtime(state, client, &id)?;
        }
        store(lkjmc_store::instance::update_desired_state(
            client, &id, "stopped",
        ))?;
        if !force && !store(lkjmc_store::temporary::cleanup_due(client, &id))? {
            return Err("temporary retention has not elapsed".to_string());
        }
        store(lkjmc_store::temporary::update_instance_state(
            client, &id, "cleaning", None,
        ))?;
        let final_state = match cleanup_files(state, &id, &temp) {
            Ok(state) => state,
            Err(error) => {
                fail_cleanup(client, &id, &error)?;
                return Err(error);
            }
        };
        store(lkjmc_store::instance::release_ports(client, &id))?;
        store(lkjmc_store::temporary::update_instance_state(
            client,
            &id,
            final_state,
            None,
        ))?;
        store(lkjmc_store::temporary::record_cleanup_event(
            client,
            Uuid::new_v4(),
            &id,
            "cleanup",
            "succeeded",
            None,
        ))?;
        audit(
            client,
            &envelope,
            "temporary.instance.cleanup",
            "temporary-instance",
            &id,
            "succeeded",
        )?;
        Ok(api::ok(
            envelope,
            json!({"id": id, "lifecycleState": final_state}),
        ))
    })
}

fn cleanup_files(
    state: &AppState,
    id: &str,
    temp: &lkjmc_store::temporary::TemporaryInstanceRecord,
) -> Result<&'static str, String> {
    let final_state = cleanup_world(&temp.world_path, &temp.cleanup_policy)?;
    let instance_root = Path::new(&state.data_root()).join(id);
    if instance_root.exists() {
        fs::remove_dir_all(&instance_root)
            .map_err(|error| format!("delete instance files: {error}"))?;
    }
    Ok(final_state)
}

fn fail_cleanup(client: &mut postgres::Client, id: &str, error: &str) -> Result<(), String> {
    store(lkjmc_store::temporary::update_instance_state(
        client,
        id,
        "failed",
        Some(error),
    ))?;
    store(lkjmc_store::temporary::record_cleanup_event(
        client,
        Uuid::new_v4(),
        id,
        "cleanup",
        "failed",
        Some(error),
    ))
}

fn cleanup_world(path: &str, policy: &str) -> Result<&'static str, String> {
    match policy {
        "delete" => {
            if Path::new(path).exists() {
                fs::remove_dir_all(path).map_err(|error| format!("delete world: {error}"))?;
            }
            Ok("cleaned")
        }
        "archive" => {
            if Path::new(path).exists() {
                let target = format!("{path}.archive.{}", Uuid::new_v4());
                fs::rename(path, &target).map_err(|error| format!("archive world: {error}"))?;
            }
            Ok("archived")
        }
        other => Err(format!("unsupported cleanup policy: {other}")),
    }
}
