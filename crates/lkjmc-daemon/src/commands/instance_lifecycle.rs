use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::{json, Value};

use crate::app::AppState;
use crate::commands::instance_create;
use crate::dispatch as api;
use crate::support::audit_helpers::audit;
use crate::support::instance_helpers::{body_string, store};

pub fn create(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let result = (|| {
        let prepared = {
            let mut client = state.database_connection()?;
            instance_create::prepare(&mut client, &request.body)?
        };
        let id = prepared.id;
        let mut config = prepared.config;
        if let Some(rcon) = request.body.get("rcon") {
            config["rcon"] = crate::runtime::rcon::private_config(&state.config_root(), &id, rcon)?;
        }
        let mut client = state.database_connection()?;
        store(lkjmc_store::instance::insert(
            &mut *client,
            &id,
            None,
            &prepared.kind,
            "stopped",
            &config,
        ))?;
        if let Err(error) =
            instance_create::assign_server_port(&mut client, &id, &request.body, &mut config)
        {
            let _ = lkjmc_store::instance::delete(&mut client, &id);
            return Err(error);
        }
        store(lkjmc_store::instance::update_config(
            &mut client,
            &id,
            &config,
        ))?;
        if let Some(asset_id) = prepared.jar_asset_id {
            store(lkjmc_store::instance::set_jar_asset(
                &mut *client,
                &id,
                asset_id,
            ))?;
        }
        audit(
            &mut *client,
            &request,
            "instance.create",
            "instance",
            &id,
            "succeeded",
        )?;
        Ok(api::ok(
            request.clone(),
            json!({
                "id":id,"desiredState":"stopped","createPlan":prepared.diagnostics
            }),
        ))
    })();
    result.unwrap_or_else(|error| api::error(request, "instance.error", error, false))
}

pub fn start(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    lifecycle_response(state, request, "running", "instance.start", |state, id| {
        reconcile(state, id, crate::runtime::RuntimeGoal::Running).map(|_| ())
    })
}

pub fn stop(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    lifecycle_response(state, request, "stopped", "instance.stop", |state, id| {
        reconcile(state, id, crate::runtime::RuntimeGoal::Stopped).map(|_| ())
    })
}

pub fn restart(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let id = match body_string(&request.body, "id") {
        Ok(id) => id,
        Err(error) => return api::error(request, "instance.error", error, false),
    };
    let result: Result<CommandResponse, String> = (|| {
        set_desired(state, &id, "stopped")?;
        reconcile(state, &id, crate::runtime::RuntimeGoal::Stopped)?;
        set_desired(state, &id, "running")?;
        reconcile(state, &id, crate::runtime::RuntimeGoal::Running)?;
        audit_result(state, &request, "instance.restart", &id)?;
        Ok(api::ok(
            request.clone(),
            json!({"id":id,"desiredState":"running"}),
        ))
    })();
    result.unwrap_or_else(|error| api::error(request, "instance.error", error, false))
}

pub fn delete(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let id = match body_string(&request.body, "id") {
        Ok(id) => id,
        Err(error) => return api::error(request, "instance.error", error, false),
    };
    let force = request
        .body
        .get("force")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let result = (|| {
        let running = reconcile(state, &id, crate::runtime::RuntimeGoal::Observe)?.healthy;
        let active = {
            let mut client = state.database_connection()?;
            store(lkjmc_store::player_session::active_count_for_server(
                &mut client,
                &id,
            ))?
        };
        if running && !force {
            return Err("instance is running; pass force to delete".to_string());
        }
        if active > 0 && !force {
            return Err(format!("instance has {active} active player session(s)"));
        }
        reconcile(state, &id, crate::runtime::RuntimeGoal::Deleted)?;
        let mut client = state.database_connection()?;
        store(lkjmc_store::instance::delete(&mut client, &id))?;
        audit(
            &mut *client,
            &request,
            "instance.delete",
            "instance",
            &id,
            "succeeded",
        )?;
        Ok(api::ok(request.clone(), json!({"id":id,"deleted":true})))
    })();
    result.unwrap_or_else(|error| api::error(request, "instance.error", error, false))
}

fn lifecycle_response(
    state: &AppState,
    request: CommandEnvelope,
    desired: &'static str,
    action: &'static str,
    effect: impl FnOnce(&AppState, &str) -> Result<(), String>,
) -> CommandResponse {
    let id = match body_string(&request.body, "id") {
        Ok(id) => id,
        Err(error) => return api::error(request, "instance.error", error, false),
    };
    let result: Result<CommandResponse, String> = (|| {
        set_desired(state, &id, desired)?;
        effect(state, &id)?;
        audit_result(state, &request, action, &id)?;
        Ok(api::ok(
            request.clone(),
            json!({"id":id,"desiredState":desired}),
        ))
    })();
    result.unwrap_or_else(|error| api::error(request, "instance.error", error, false))
}

fn set_desired(state: &AppState, id: &str, desired: &str) -> Result<(), String> {
    let mut client = state.database_connection()?;
    store(lkjmc_store::instance_presence::clear_autosuspended(
        &mut client,
        id,
    ))?;
    store(lkjmc_store::instance::update_desired_state(
        &mut client,
        id,
        desired,
    ))?;
    Ok(())
}

fn reconcile(
    state: &AppState,
    id: &str,
    goal: crate::runtime::RuntimeGoal,
) -> Result<crate::runtime::RuntimeObservation, String> {
    crate::runtime::reconcile::reconcile(state, id, goal, uuid::Uuid::new_v4())
}

fn audit_result(
    state: &AppState,
    request: &CommandEnvelope,
    action: &str,
    id: &str,
) -> Result<(), String> {
    let mut client = state.database_connection()?;
    audit(&mut *client, request, action, "instance", id, "succeeded")
}
