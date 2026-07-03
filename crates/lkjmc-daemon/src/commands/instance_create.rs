use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use lkjmc_core::instance_create::{plan_startable, CreatePlanInput, LaunchSource};
use postgres::Client;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;
use crate::commands::instance_create_assets::jar_asset;
use crate::commands::instance_create_diagnostics::{failure, plan_failure, PlanFailure};
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, create_config, store, with_connection};

pub struct PreparedCreate {
    pub id: String,
    pub kind: String,
    pub config: Value,
    pub jar_asset_id: Option<Uuid>,
    pub diagnostics: Value,
}

pub fn plan(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    with_connection(
        state,
        request,
        |_state, request, client| match prepare_checked(client, &request.body) {
            Ok(prepared) => Ok(api::ok(
                request,
                json!({
                    "startable": true,
                    "id": prepared.id,
                    "kind": prepared.kind,
                    "createPlan": prepared.diagnostics,
                    "diagnostics": []
                }),
            )),
            Err(error) => Ok(api::ok(
                request,
                json!({
                    "startable": false,
                    "diagnostic": error.diagnostic,
                    "diagnostics": [error.diagnostic]
                }),
            )),
        },
    )
}

pub fn prepare(client: &mut Client, body: &Value) -> Result<PreparedCreate, String> {
    prepare_checked(client, body).map_err(|error| error.message)
}

fn prepare_checked(client: &mut Client, body: &Value) -> Result<PreparedCreate, PlanFailure> {
    let id = body_string(body, "id").map_err(|e| failure("invalid_request", &e, json!({})))?;
    let kind = body_string(body, "kind").map_err(|e| failure("invalid_request", &e, json!({})))?;
    let template =
        body_string(body, "template").map_err(|e| failure("invalid_request", &e, json!({})))?;
    let jar = jar_asset(client, body, &kind, &template).map_err(|e| {
        failure(
            "jar_registry_error",
            &e,
            json!({"kind": kind, "template": template}),
        )
    })?;
    if let Some(asset_id) = jar.missing_explicit {
        return Err(failure(
            "jar_asset_not_found",
            &format!("Jar asset was not found: {asset_id}"),
            json!({"jarAssetId": asset_id.to_string()}),
        ));
    }
    let jar_asset_id = jar.asset_id;
    let launch_source = launch_source(body, jar_asset_id);
    let plan = plan_startable(CreatePlanInput {
        id: id.clone(),
        kind: kind.clone(),
        template: template.clone(),
        launch_source,
        memory_mb: body.get("memoryMb").and_then(Value::as_i64),
        server_port: body.get("serverPort").and_then(Value::as_i64),
        accept_minecraft_eula: eula_accepted(body),
    })
    .map_err(|errors| plan_failure(errors, &kind, &template, jar.attempted_queries.clone()))?;
    let mut config = create_config(body, &template);
    config["memoryMb"] = Value::Number(plan.memory_mb.into());
    config["eulaAccepted"] = Value::Bool(plan.eula_accepted);
    if let Some(asset_id) = jar_asset_id {
        config["jarAssetId"] = Value::String(asset_id.to_string());
    }
    Ok(PreparedCreate {
        id,
        kind,
        config,
        jar_asset_id,
        diagnostics: json!({
            "template": template,
            "memoryMb": plan.memory_mb,
            "serverPort": plan.server_port,
            "jarAssetId": jar_asset_id.map(|value| value.to_string()),
            "launchSource": launch_label(&plan.launch_source),
            "eulaAccepted": plan.eula_accepted
        }),
    })
}

pub fn assign_server_port(
    client: &mut Client,
    id: &str,
    body: &Value,
    config: &mut Value,
) -> Result<i32, String> {
    let port = match body.get("serverPort").and_then(Value::as_i64) {
        Some(port) => {
            let port = i32::try_from(port).map_err(|error| error.to_string())?;
            store(lkjmc_store::instance::reserve_port(
                client, id, port, "server",
            ))?;
            port
        }
        None => store(lkjmc_store::instance::allocate_port(
            client, id, "server", 25565, 25665,
        ))?,
    };
    config["serverPort"] = Value::Number(i64::from(port).into());
    Ok(port)
}

fn launch_source(body: &Value, jar_asset_id: Option<Uuid>) -> Option<LaunchSource> {
    if let Some(asset_id) = jar_asset_id {
        Some(LaunchSource::JarAsset(asset_id.to_string()))
    } else if body.get("command").and_then(Value::as_str).is_some() {
        Some(LaunchSource::Command)
    } else {
        None
    }
}

fn eula_accepted(body: &Value) -> bool {
    body.get("acceptMinecraftEula")
        .or_else(|| body.get("eulaAccepted"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn launch_label(source: &LaunchSource) -> &'static str {
    match source {
        LaunchSource::JarAsset(_) => "jar-asset",
        LaunchSource::Command => "command",
    }
}
