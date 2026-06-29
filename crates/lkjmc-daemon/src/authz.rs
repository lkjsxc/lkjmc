use lkjmc_core::command::{ActorKind, CommandEnvelope, CommandResponse};

use crate::api;
use crate::app::AppState;

pub fn required(command: &str) -> Option<&'static str> {
    match command {
        "status" | "doctor" => Some("lkjmc.admin.status"),
        "config.reload" => Some("lkjmc.admin.reload"),
        "instance.list" => Some("lkjmc.admin.instance.list"),
        "instance.create" => Some("lkjmc.admin.instance.create"),
        "instance.start" => Some("lkjmc.admin.instance.start"),
        "instance.stop" => Some("lkjmc.admin.instance.stop"),
        "instance.restart" => Some("lkjmc.admin.instance.restart"),
        "instance.delete" => Some("lkjmc.admin.instance.delete"),
        "economy.catalog.seed-defaults" | "shop.item.upsert" => Some("lkjmc.admin.economy"),
        "admin.grant.create" | "admin.grant.revoke" | "admin.audit.tail" => {
            Some("lkjmc.admin.admin")
        }
        _ => None,
    }
}

pub fn enforce(
    state: &AppState,
    request: &CommandEnvelope,
    permission: &str,
) -> Option<CommandResponse> {
    if request.actor.kind == ActorKind::Cli || platform_allowed(request) {
        return None;
    }
    if grant_allowed(state, request, permission).unwrap_or(false) {
        return None;
    }
    Some(api::error(
        request.clone(),
        "admin.denied",
        "admin permission denied",
        false,
    ))
}

fn platform_allowed(request: &CommandEnvelope) -> bool {
    request
        .body
        .get("platformPermission")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

fn grant_allowed(
    state: &AppState,
    request: &CommandEnvelope,
    permission: &str,
) -> Result<bool, String> {
    let Some(database_url) = state.database_url() else {
        return Ok(false);
    };
    let kind = request
        .body
        .get("principalKind")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("minecraft-player");
    let Some(id) = request
        .body
        .get("principalId")
        .and_then(serde_json::Value::as_str)
    else {
        return Ok(false);
    };
    let mut client =
        lkjmc_store::pool::connect(&database_url).map_err(|error| error.to_string())?;
    let permissions = lkjmc_store::admin::effective_permissions(&mut client, kind, id)
        .map_err(|error| error.to_string())?;
    if permissions
        .iter()
        .any(|value| value == permission || value == "lkjmc.admin.admin")
    {
        return Ok(true);
    }
    let _ = lkjmc_store::admin::insert_audit(
        &mut client,
        "daemon",
        "authz",
        "admin.denied",
        kind,
        id,
        "denied",
    );
    Ok(false)
}
