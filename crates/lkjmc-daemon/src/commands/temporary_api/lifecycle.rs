use lkjmc_core::bootstrap::PluginId;
use serde_json::json;

use crate::app::AppState;
use crate::commands::temporary_api::{readiness, request};
use crate::dispatch as api;
use crate::support::audit_helpers::audit;
use crate::support::instance_helpers::{
    body_string, start_runtime, stop_runtime, store, with_connection,
};

pub fn start(
    state: &AppState,
    envelope: lkjmc_core::command::CommandEnvelope,
) -> lkjmc_core::command::CommandResponse {
    let id = match body_string(&envelope.body, "id") {
        Ok(id) => id,
        Err(error) => return api::error(envelope, "temporary.error", error, false),
    };
    match start_ready(state, &id, timeout(&envelope.body)) {
        Ok(()) => match state.database_connection().and_then(|mut client| {
            audit(
                &mut *client,
                &envelope,
                "temporary.instance.start",
                "temporary-instance",
                &id,
                "succeeded",
            )
        }) {
            Ok(()) => api::ok(envelope, json!({"id": id, "lifecycleState": "ready"})),
            Err(error) => api::error(envelope, "temporary.error", error, false),
        },
        Err(error) => {
            if let Ok(mut client) = state.database_connection() {
                let _ = audit(
                    &mut *client,
                    &envelope,
                    "temporary.instance.start",
                    "temporary-instance",
                    &id,
                    "failed",
                );
            }
            api::error(envelope, "temporary.error", error, false)
        }
    }
}

pub(crate) fn start_ready(state: &AppState, id: &str, timeout_seconds: u64) -> Result<(), String> {
    let port = {
        let mut client = state.database_connection()?;
        let temp = require_temp(&mut client, id)?;
        if !matches!(temp.lifecycle_state.as_str(), "created" | "stopped") {
            return Err(format!(
                "temporary instance cannot start from {}",
                temp.lifecycle_state
            ));
        }
        let port = readiness::server_port(&mut client, id)?;
        store(lkjmc_store::temporary::update_instance_state(
            &mut *client,
            id,
            "starting",
            None,
        ))?;
        crate::assets::plugin_install::install(state, &mut client, id, PluginId::LkjmcPaper)?;
        store(lkjmc_store::instance::update_desired_state(
            &mut client,
            id,
            "running",
        ))?;
        drop(client);
        if let Err(error) = start_runtime(state, id) {
            let mut client = state.database_connection()?;
            return mark_start_failed(&mut client, id, error);
        }
        port
    };
    let result = readiness::wait_ready(state, id, port, timeout_seconds);
    let mut client = state.database_connection()?;
    match result {
        Ok(()) => store(lkjmc_store::temporary::update_instance_state(
            &mut *client,
            id,
            "ready",
            None,
        )),
        Err(error) => mark_start_failed(&mut client, id, error),
    }
}

pub fn stop(
    state: &AppState,
    envelope: lkjmc_core::command::CommandEnvelope,
) -> lkjmc_core::command::CommandResponse {
    let id = match body_string(&envelope.body, "id") {
        Ok(id) => id,
        Err(error) => return api::error(envelope, "temporary.error", error, false),
    };
    let result: Result<lkjmc_core::command::CommandResponse, String> = (|| {
        {
            let mut client = state.database_connection()?;
            require_temp(&mut client, &id)?;
            store(lkjmc_store::temporary::update_instance_state(
                &mut *client,
                &id,
                "stopping",
                None,
            ))?;
            store(lkjmc_store::instance::update_desired_state(
                &mut client,
                &id,
                "stopped",
            ))?;
        }
        stop_runtime(state, &id)?;
        let mut client = state.database_connection()?;
        store(lkjmc_store::temporary::update_instance_state(
            &mut *client,
            &id,
            "stopped",
            None,
        ))?;
        audit(
            &mut *client,
            &envelope,
            "temporary.instance.stop",
            "temporary-instance",
            &id,
            "succeeded",
        )?;
        Ok(api::ok(
            envelope.clone(),
            json!({"id":id,"lifecycleState":"stopped"}),
        ))
    })();
    result.unwrap_or_else(|error| api::error(envelope, "temporary.error", error, false))
}

pub fn get(
    state: &AppState,
    envelope: lkjmc_core::command::CommandEnvelope,
) -> lkjmc_core::command::CommandResponse {
    with_connection(state, envelope, |_state, envelope, client| {
        let id = body_string(&envelope.body, "id")?;
        let temp = require_temp(client, &id)?;
        Ok(api::ok(
            envelope,
            json!({
                "id": temp.instance_id, "ownerKind": temp.owner_kind, "ownerId": temp.owner_id,
                "lifecycleState": temp.lifecycle_state, "cleanupPolicy": temp.cleanup_policy,
                "worldPath": temp.world_path, "serverPort": temp.server_port,
                "expiresInSeconds": temp.expires_in_seconds
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

fn mark_start_failed(client: &mut postgres::Client, id: &str, error: String) -> Result<(), String> {
    store(lkjmc_store::temporary::update_instance_state(
        client,
        id,
        "failed",
        Some(&error),
    ))?;
    store(lkjmc_store::instance::update_desired_state(
        client, id, "failed",
    ))?;
    Err(error)
}

fn timeout(body: &serde_json::Value) -> u64 {
    u64::from(request::u32_field(body, "readinessTimeoutSeconds", 180).unwrap_or(180))
}
