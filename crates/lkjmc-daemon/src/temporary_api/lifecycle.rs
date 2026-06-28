use lkjmc_core::bootstrap::PluginId;
use serde_json::json;

use crate::api;
use crate::app::AppState;
use crate::audit_helpers::audit;
use crate::instance_helpers::{body_string, start_runtime, stop_runtime, store, with_client};
use crate::temporary_api::{readiness, request};

pub fn start(
    state: &AppState,
    envelope: lkjmc_core::command::CommandEnvelope,
) -> lkjmc_core::command::CommandResponse {
    with_client(state, envelope, |state, envelope, client| {
        let id = body_string(&envelope.body, "id")?;
        let temp = require_temp(client, &id)?;
        if !matches!(temp.lifecycle_state.as_str(), "created" | "stopped") {
            return Err(format!(
                "temporary instance cannot start from {}",
                temp.lifecycle_state
            ));
        }
        store(lkjmc_store::temporary::update_instance_state(
            client, &id, "starting", None,
        ))?;
        crate::plugin_install::install(state, client, &id, PluginId::LkjmcPaper)?;
        store(lkjmc_store::instance::update_desired_state(
            client, &id, "running",
        ))?;
        let result = start_runtime(state, client, &id).and_then(|observation| {
            if observation.healthy {
                readiness::wait_ready(state, client, &id, timeout(&envelope.body))
            } else {
                Err(observation
                    .message
                    .unwrap_or_else(|| "temporary start failed".to_string()))
            }
        });
        match result {
            Ok(()) => {
                store(lkjmc_store::temporary::update_instance_state(
                    client, &id, "ready", None,
                ))?;
                audit(
                    client,
                    &envelope,
                    "temporary.instance.start",
                    "temporary-instance",
                    &id,
                    "succeeded",
                )?;
                Ok(api::ok(
                    envelope,
                    json!({"id": id, "lifecycleState": "ready"}),
                ))
            }
            Err(error) => fail_start(client, envelope, &id, error),
        }
    })
}

pub fn stop(
    state: &AppState,
    envelope: lkjmc_core::command::CommandEnvelope,
) -> lkjmc_core::command::CommandResponse {
    with_client(state, envelope, |state, envelope, client| {
        let id = body_string(&envelope.body, "id")?;
        require_temp(client, &id)?;
        store(lkjmc_store::temporary::update_instance_state(
            client, &id, "stopping", None,
        ))?;
        store(lkjmc_store::instance::update_desired_state(
            client, &id, "stopped",
        ))?;
        stop_runtime(state, client, &id)?;
        store(lkjmc_store::temporary::update_instance_state(
            client, &id, "stopped", None,
        ))?;
        audit(
            client,
            &envelope,
            "temporary.instance.stop",
            "temporary-instance",
            &id,
            "succeeded",
        )?;
        Ok(api::ok(
            envelope,
            json!({"id": id, "lifecycleState": "stopped"}),
        ))
    })
}

pub fn get(
    state: &AppState,
    envelope: lkjmc_core::command::CommandEnvelope,
) -> lkjmc_core::command::CommandResponse {
    with_client(state, envelope, |_state, envelope, client| {
        let id = body_string(&envelope.body, "id")?;
        let temp = require_temp(client, &id)?;
        Ok(api::ok(
            envelope,
            json!({
                "id": temp.instance_id,
                "ownerKind": temp.owner_kind,
                "ownerId": temp.owner_id,
                "lifecycleState": temp.lifecycle_state,
                "cleanupPolicy": temp.cleanup_policy,
                "worldPath": temp.world_path,
                "serverPort": temp.server_port
            }),
        ))
    })
}

pub(super) fn require_temp(
    client: &mut postgres::Client,
    id: &str,
) -> Result<lkjmc_store::temporary::TemporaryInstanceRecord, String> {
    store(lkjmc_store::temporary::get_instance(client, id))?
        .ok_or_else(|| format!("temporary instance not found: {id}"))
}

fn fail_start(
    client: &mut postgres::Client,
    envelope: lkjmc_core::command::CommandEnvelope,
    id: &str,
    error: String,
) -> Result<lkjmc_core::command::CommandResponse, String> {
    store(lkjmc_store::temporary::update_instance_state(
        client,
        id,
        "failed",
        Some(&error),
    ))?;
    store(lkjmc_store::instance::update_desired_state(
        client, id, "failed",
    ))?;
    audit(
        client,
        &envelope,
        "temporary.instance.start",
        "temporary-instance",
        id,
        "failed",
    )?;
    Err(error)
}

fn timeout(body: &serde_json::Value) -> u64 {
    u64::from(request::u32_field(body, "readinessTimeoutSeconds", 180).unwrap_or(180))
}
