use lkjmc_core::command::CommandEnvelope;
use serde_json::{json, Value};

use crate::app::AppState;
use crate::commands::instance_create;
use crate::dispatch as api;
use crate::support::audit_helpers::audit;
use crate::support::instance_helpers::*;

pub fn create(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    with_connection(state, request, |_state, request, client| {
        let prepared = instance_create::prepare(client, &request.body)?;
        let id = prepared.id;
        let mut config = prepared.config;
        store(lkjmc_store::instance::insert(
            client,
            &id,
            None,
            &prepared.kind,
            "stopped",
            &config,
        ))?;
        if let Err(error) =
            instance_create::assign_server_port(client, &id, &request.body, &mut config)
        {
            let _ = lkjmc_store::instance::delete(client, &id);
            return Err(error);
        }
        store(lkjmc_store::instance::update_config(client, &id, &config))?;
        if let Some(asset_id) = prepared.jar_asset_id {
            store(lkjmc_store::instance::set_jar_asset(client, &id, asset_id))?;
        }
        audit(
            client,
            &request,
            "instance.create",
            "instance",
            &id,
            "succeeded",
        )?;
        Ok(api::ok(
            request,
            json!({"id": id, "desiredState": "stopped", "createPlan": prepared.diagnostics}),
        ))
    })
}

pub fn start(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    with_connection(state, request, |state, request, client| {
        let id = body_string(&request.body, "id")?;
        start_instance(state, client, &id)?;
        audit(
            client,
            &request,
            "instance.start",
            "instance",
            &id,
            "succeeded",
        )?;
        Ok(api::ok(
            request,
            json!({"id": id, "desiredState": "running"}),
        ))
    })
}

pub fn stop(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    with_connection(state, request, |state, request, client| {
        let id = body_string(&request.body, "id")?;
        stop_instance(state, client, &id)?;
        audit(
            client,
            &request,
            "instance.stop",
            "instance",
            &id,
            "succeeded",
        )?;
        Ok(api::ok(
            request,
            json!({"id": id, "desiredState": "stopped"}),
        ))
    })
}

pub fn restart(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    with_connection(state, request, |state, request, client| {
        let id = body_string(&request.body, "id")?;
        stop_runtime(state, client, &id)?;
        start_instance(state, client, &id)?;
        audit(
            client,
            &request,
            "instance.restart",
            "instance",
            &id,
            "succeeded",
        )?;
        Ok(api::ok(
            request,
            json!({"id": id, "desiredState": "running"}),
        ))
    })
}

pub fn delete(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    with_connection(state, request, |state, request, client| {
        let id = body_string(&request.body, "id")?;
        let force = request
            .body
            .get("force")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if runtime_running(state, &id)? && !force {
            return Err("instance is running; pass force to delete".to_string());
        }
        let active = store(lkjmc_store::player_session::active_count_for_server(
            client, &id,
        ))?;
        if active > 0 && !force {
            return Err(format!("instance has {active} active player session(s)"));
        }
        if force {
            stop_runtime(state, client, &id)?;
        }
        {
            let mut runtime = state
                .runtime
                .lock()
                .map_err(|_| "runtime lock poisoned".to_string())?;
            let _ = runtime.delete(&id)?;
        }
        store(lkjmc_store::instance::delete(client, &id))?;
        audit(
            client,
            &request,
            "instance.delete",
            "instance",
            &id,
            "succeeded",
        )?;
        Ok(api::ok(request, json!({"id": id, "deleted": true})))
    })
}

fn start_instance(state: &AppState, client: &mut postgres::Client, id: &str) -> Result<(), String> {
    if runtime_running(state, id)? {
        store(lkjmc_store::instance::update_desired_state(
            client, id, "running",
        ))?;
        return Ok(());
    }
    let previous = store(lkjmc_store::instance::get(client, id))?
        .map(|record| record.desired_state)
        .unwrap_or_else(|| "stopped".to_string());
    store(lkjmc_store::instance_presence::clear_autosuspended(
        client, id,
    ))?;
    match start_runtime(state, client, id) {
        Ok(_) => {
            store(lkjmc_store::instance::update_desired_state(
                client, id, "running",
            ))?;
            Ok(())
        }
        Err(error) => {
            let _ = lkjmc_store::instance::update_desired_state(client, id, &previous);
            let _ = lkjmc_store::instance::upsert_observation(
                client,
                id,
                "process-unhealthy",
                None,
                false,
                Some(&error),
            );
            Err(error)
        }
    }
}

fn stop_instance(state: &AppState, client: &mut postgres::Client, id: &str) -> Result<(), String> {
    store(lkjmc_store::instance::update_desired_state(
        client, id, "stopped",
    ))?;
    stop_runtime(state, client, id).map(|_| ())
}
