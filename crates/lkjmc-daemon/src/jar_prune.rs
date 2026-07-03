use std::fs;
use std::path::Path;

use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::json;

use crate::api;
use crate::app::AppState;
use crate::audit_helpers::audit;

pub fn handle(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    match prune(state, request.clone()) {
        Ok(count) => api::ok(request, json!({"pruned": count})),
        Err(error) => api::error(request, "jar.prune_failed", error, false),
    }
}

fn prune(state: &AppState, request: CommandEnvelope) -> Result<usize, String> {
    if request.body.get("yes").and_then(|value| value.as_bool()) != Some(true) {
        return Err("jar prune requires yes".to_string());
    }
    if state.database_url().is_none() {
        return Err("Database URL is not configured".to_string());
    }
    let mut client = state.database_connection()?;
    let assets = lkjmc_store::jar::prunable(&mut client).map_err(|error| error.to_string())?;
    let mut count = 0_usize;
    for asset in assets {
        remove_file(&asset.path)?;
        lkjmc_store::jar::delete(&mut client, asset.id).map_err(|error| error.to_string())?;
        audit(
            &mut *client,
            &request,
            "jar.prune",
            "jar_asset",
            &asset.id.to_string(),
            "succeeded",
        )?;
        count += 1;
    }
    Ok(count)
}

fn remove_file(path: &str) -> Result<(), String> {
    let path = Path::new(path);
    if !path.exists() {
        return Ok(());
    }
    fs::remove_file(path).map_err(|error| format!("remove jar {}: {error}", path.display()))
}
