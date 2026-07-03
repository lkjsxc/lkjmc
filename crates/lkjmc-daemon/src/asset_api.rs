use lkjmc_core::bootstrap::PluginId;
use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use lkjmc_core::id::CommandId;
use serde_json::{json, Value};

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::body_string;

pub fn handle(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    match request.command.as_str() {
        "asset.server.sync" => server_sync(state, request),
        "asset.plugin.sync" => plugin_sync(state, request),
        "asset.plugin.list" => plugin_list(state, request),
        "asset.plugin.inspect" => plugin_inspect(state, request),
        _ => api::error(request, "command.unknown", "unknown asset command", false),
    }
}

fn server_sync(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let project = match body_string(&request.body, "project") {
        Ok(value) => value,
        Err(error) => return api::error(request, "asset.request", error, false),
    };
    let mut body = json!({"project": project, "channel": "stable"});
    if let Some(release) = request.body.get("minecraftRelease").and_then(Value::as_str) {
        body["minecraftRelease"] = json!(release);
    }
    crate::downloads::handle(
        state,
        CommandEnvelope {
            request_id: CommandId::internal("asset-server-sync"),
            actor: request.actor,
            command: "jar.sync".to_string(),
            body,
        },
    )
}

fn plugin_sync(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    with_connection(state, request, |state, request, client| {
        let plugin = plugin_from_body(&request.body)?;
        let asset = match plugin {
            PluginId::LkjmcPaper | PluginId::LkjmcVelocity => {
                crate::plugin_assets::register_local(state, client, plugin)?
            }
            _ => crate::plugin_downloads::sync(state, client, plugin)?,
        };
        Ok(api::ok(request, asset_json(asset)))
    })
}

fn plugin_list(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    with_connection(state, request, |_state, request, client| {
        let assets = lkjmc_store::asset::list(client)
            .map_err(|error| error.to_string())?
            .into_iter()
            .filter(|asset| asset.asset_kind == "plugin")
            .map(asset_json)
            .collect::<Vec<Value>>();
        Ok(api::ok(request, json!({"plugins": assets})))
    })
}

fn plugin_inspect(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    with_connection(state, request, |_state, request, client| {
        let plugin = plugin_from_body(&request.body)?;
        let asset = lkjmc_store::asset::latest_for_project(client, "plugin", plugin.as_str())
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("plugin asset not found: {}", plugin.as_str()))?;
        Ok(api::ok(request, asset_json(asset)))
    })
}

fn with_connection<F>(state: &AppState, request: CommandEnvelope, action: F) -> CommandResponse
where
    F: FnOnce(&AppState, CommandEnvelope, &mut postgres::Client) -> Result<CommandResponse, String>,
{
    let Some(_database_url) = state.database_url() else {
        return api::error(
            request,
            "database.not_configured",
            "Database URL is not configured",
            false,
        );
    };
    let mut client = match state.database_connection() {
        Ok(client) => client,
        Err(error) => return api::error(request, "database.error", error, false),
    };
    match action(state, request.clone(), &mut client) {
        Ok(response) => response,
        Err(error) => api::error(request, "asset.error", error, false),
    }
}

fn plugin_from_body(body: &Value) -> Result<PluginId, String> {
    let value = body_string(body, "plugin")?;
    match value.as_str() {
        "lkjmc-paper" => Ok(PluginId::LkjmcPaper),
        "lkjmc-velocity" => Ok(PluginId::LkjmcVelocity),
        "viaversion" => Ok(PluginId::ViaVersion),
        "viabackwards" => Ok(PluginId::ViaBackwards),
        "geyser" => Ok(PluginId::Geyser),
        "floodgate" => Ok(PluginId::Floodgate),
        _ => Err(format!("unknown plugin: {value}")),
    }
}

fn asset_json(asset: lkjmc_store::asset::AssetRecord) -> Value {
    json!({
        "id": asset.id.to_string(),
        "kind": asset.asset_kind,
        "platform": asset.platform,
        "project": asset.project,
        "channel": asset.channel,
        "name": asset.name,
        "fileName": asset.file_name,
        "path": asset.path,
        "sha256": asset.sha256,
        "sizeBytes": asset.size_bytes,
        "source": asset.source
    })
}
