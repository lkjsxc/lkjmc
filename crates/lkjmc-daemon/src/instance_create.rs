use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use lkjmc_core::instance_create::{plan_startable, CreatePlanInput, LaunchSource};
use postgres::Client;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, create_config, store, with_connection};

pub struct PreparedCreate {
    pub id: String,
    pub kind: String,
    pub config: Value,
    pub jar_asset_id: Option<Uuid>,
    pub diagnostics: Value,
}

pub fn plan(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    with_connection(state, request, |_state, request, client| {
        match prepare(client, &request.body) {
            Ok(prepared) => Ok(api::ok(
                request,
                json!({
                    "startable": true,
                    "id": prepared.id,
                    "kind": prepared.kind,
                    "createPlan": prepared.diagnostics
                }),
            )),
            Err(error) => Ok(api::ok(
                request,
                json!({
                    "startable": false,
                    "diagnostics": [error]
                }),
            )),
        }
    })
}

pub fn prepare(client: &mut Client, body: &Value) -> Result<PreparedCreate, String> {
    let id = body_string(body, "id")?;
    let kind = body_string(body, "kind")?;
    let template = body_string(body, "template")?;
    let jar_asset_id = jar_asset(client, body, &kind, &template)?;
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
    .map_err(|errors| errors.join("; "))?;
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

fn jar_asset(
    client: &mut Client,
    body: &Value,
    kind: &str,
    template: &str,
) -> Result<Option<Uuid>, String> {
    if let Some(asset_id) = body.get("jarAssetId").and_then(Value::as_str) {
        let asset_id = Uuid::parse_str(asset_id).map_err(|error| error.to_string())?;
        store(lkjmc_store::jar::get(client, asset_id))?
            .ok_or_else(|| format!("jar asset not found: {asset_id}"))?;
        return Ok(Some(asset_id));
    }
    if body.get("command").and_then(Value::as_str).is_some() {
        return Ok(None);
    }
    default_asset(client, kind, template)
}

fn default_asset(client: &mut Client, kind: &str, template: &str) -> Result<Option<Uuid>, String> {
    for query in asset_queries(kind, template) {
        if let Some(asset) = store(lkjmc_store::jar::latest_matching(client, &query))? {
            return Ok(Some(asset.id));
        }
    }
    Ok(None)
}

fn asset_queries(kind: &str, template: &str) -> Vec<String> {
    let mut queries = vec![kind.to_string(), template.to_string()];
    if let Some(prefix) = template.split('-').next() {
        if !queries.iter().any(|value| value == prefix) {
            queries.push(prefix.to_string());
        }
    }
    queries
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
