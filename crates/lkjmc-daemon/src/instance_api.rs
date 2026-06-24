use lkjmc_core::command::CommandEnvelope;
use lkjmc_core::id::InstanceId;
use serde_json::{json, Value};

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::*;

pub fn handle(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    match request.command.as_str() {
        "instance.list" => list(state, request),
        "instance.create" => create(state, request),
        "instance.start" => start(state, request),
        "instance.stop" => stop(state, request),
        "instance.restart" => restart(state, request),
        "instance.delete" => delete(state, request),
        "instance.logs" => logs(state, request),
        _ => api::error(
            request,
            "command.unknown",
            "unknown instance command",
            false,
        ),
    }
}

fn list(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    with_client(state, request, |state, request, client| {
        refresh_runtime(state, client)?;
        let instances = store(lkjmc_store::instance::list(client))?
            .into_iter()
            .map(|row| {
                json!({
                    "id": row.id,
                    "kind": row.kind,
                    "desiredState": row.desired_state,
                    "observedState": row.observed_state,
                    "healthy": row.healthy
                })
            })
            .collect::<Vec<Value>>();
        Ok(api::ok(request, json!({"instances": instances})))
    })
}

fn create(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
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

fn start(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    with_client(state, request, |state, request, client| {
        let id = body_string(&request.body, "id")?;
        let config = store(lkjmc_store::instance::config(client, &id))?
            .ok_or_else(|| format!("instance not found: {id}"))?;
        let (command, args) = launch(&config)?;
        store(lkjmc_store::instance::update_desired_state(
            client, &id, "running",
        ))?;
        let observation = runtime_start(state, &id, &command, &args)?;
        write_observation(client, &id, &observation)?;
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
            json!({"id": id, "observedState": observation.observed_state}),
        ))
    })
}

fn stop(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    with_client(state, request, |state, request, client| {
        let id = body_string(&request.body, "id")?;
        store(lkjmc_store::instance::update_desired_state(
            client, &id, "stopped",
        ))?;
        let observation = runtime_stop(state, &id)?;
        write_observation(client, &id, &observation)?;
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
            json!({"id": id, "observedState": observation.observed_state}),
        ))
    })
}

fn restart(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    with_client(state, request, |state, request, client| {
        let id = body_string(&request.body, "id")?;
        let config = store(lkjmc_store::instance::config(client, &id))?
            .ok_or_else(|| format!("instance not found: {id}"))?;
        let (command, args) = launch(&config)?;
        let stopped = runtime_stop(state, &id)?;
        write_observation(client, &id, &stopped)?;
        let started = runtime_start(state, &id, &command, &args)?;
        store(lkjmc_store::instance::update_desired_state(
            client, &id, "running",
        ))?;
        write_observation(client, &id, &started)?;
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
            json!({"id": id, "observedState": started.observed_state}),
        ))
    })
}

fn delete(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
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
        if force {
            let stopped = runtime_stop(state, &id)?;
            write_observation(client, &id, &stopped)?;
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

fn logs(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    let id = match body_string(&request.body, "id") {
        Ok(value) => value,
        Err(error) => return api::error(request, "request.invalid", error, false),
    };
    let lines = request
        .body
        .get("lines")
        .and_then(Value::as_u64)
        .unwrap_or(120)
        .min(500) as usize;
    match crate::logs::tail(&state.log_root, &id, lines) {
        Ok(lines) => api::ok(request, json!({"id": id, "lines": lines})),
        Err(error) => api::error(request, "instance.logs_failed", error, false),
    }
}
