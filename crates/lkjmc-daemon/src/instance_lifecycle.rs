use lkjmc_core::command::CommandEnvelope;
use lkjmc_core::id::InstanceId;
use serde_json::{json, Value};

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::*;

pub fn create(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    with_client(state, request, |_state, request, client| {
        let id = body_string(&request.body, "id")?;
        InstanceId::parse(id.clone()).map_err(|error| error.to_string())?;
        let kind = body_string(&request.body, "kind")?;
        let template = body_string(&request.body, "template")?;
        let config = create_config(&request.body, &template);
        store(lkjmc_store::instance::insert(
            client, &id, None, &kind, "stopped", &config,
        ))?;
        if let Some(port) = request.body.get("serverPort").and_then(Value::as_i64) {
            store(lkjmc_store::instance::reserve_port(
                client,
                &id,
                port as i32,
                "server",
            ))?;
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
            json!({"id": id, "desiredState": "stopped"}),
        ))
    })
}

pub fn start(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    with_client(state, request, |state, request, client| {
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
    with_client(state, request, |state, request, client| {
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
    with_client(state, request, |state, request, client| {
        let id = body_string(&request.body, "id")?;
        stop_runtime(state, client, &id)?;
        start_runtime(state, client, &id)?;
        store(lkjmc_store::instance::update_desired_state(
            client, &id, "running",
        ))?;
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
    with_client(state, request, |state, request, client| {
        let id = body_string(&request.body, "id")?;
        let force = request
            .body
            .get("force")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if runtime_running(state, &id)? && !force {
            return Err("instance is running; pass force to delete".to_string());
        }
        let active = store(lkjmc_store::player::active_session_count_for_server(
            client, &id,
        ))?;
        if active > 0 && !force {
            return Err(format!("instance has {active} active player session(s)"));
        }
        if force {
            stop_runtime(state, client, &id)?;
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
    store(lkjmc_store::instance::update_desired_state(
        client, id, "running",
    ))?;
    start_runtime(state, client, id).map(|_| ())
}

fn stop_instance(state: &AppState, client: &mut postgres::Client, id: &str) -> Result<(), String> {
    store(lkjmc_store::instance::update_desired_state(
        client, id, "stopped",
    ))?;
    stop_runtime(state, client, id).map(|_| ())
}
