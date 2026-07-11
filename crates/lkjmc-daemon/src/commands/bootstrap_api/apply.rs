mod effects;
mod readiness_wait;
mod steps;

use lkjmc_core::bootstrap::{BootstrapEffect, DiagnosticSeverity};
use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;

pub fn apply(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let bootstrap_request = match super::request::from_body(state, &request.body, false) {
        Ok(request) => request,
        Err(error) => return api::error(request, "bootstrap.request", error, false),
    };
    if let Err(error) = super::database_url(state) {
        return api::error(request, "bootstrap.apply_failed", error, false);
    }
    if !bootstrap_request.accept_minecraft_eula {
        return api::error(
            request,
            "bootstrap.eula_required",
            "pass --accept-minecraft-eula or set LKJMC_ACCEPT_MINECRAFT_EULA=1",
            false,
        );
    }
    let facts = crate::commands::bootstrap_facts::gather(state);
    let plan = lkjmc_core::bootstrap::plan_bootstrap(&bootstrap_request, &facts);
    if plan
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Blocking)
    {
        return api::error(request, "bootstrap.blocked", blocking_message(&plan), false);
    }
    match run_plan(
        state,
        &request,
        plan.effects,
        serde_json::to_value(plan.diagnostics),
    ) {
        Ok(body) => api::ok(request, body),
        Err(error) => api::error(request, "bootstrap.apply_failed", error, false),
    }
}

fn blocking_message(plan: &lkjmc_core::bootstrap::BootstrapPlan) -> String {
    plan.diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Blocking)
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>()
        .join("; ")
}

fn run_plan(
    state: &AppState,
    request: &CommandEnvelope,
    effects: Vec<BootstrapEffect>,
    diagnostics: Result<Value, serde_json::Error>,
) -> Result<Value, String> {
    if super::database_url(state)?.is_none() {
        return Err("Database URL is not configured".to_string());
    }
    let mut lock = acquire_apply_lock(state)?;
    let result = state
        .database_connection()
        .and_then(|client| run_effects(state, request, effects, diagnostics, client));
    let release =
        lkjmc_store::bootstrap::release_apply_lock(&mut lock).map_err(|error| error.to_string());
    match (result, release) {
        (Ok(value), Ok(())) => Ok(value),
        (Ok(_), Err(error)) | (Err(error), _) => Err(error),
    }
}

fn acquire_apply_lock(state: &AppState) -> Result<postgres::Client, String> {
    let url =
        super::database_url(state)?.ok_or_else(|| "Database URL is not configured".to_string())?;
    let mut lock = lkjmc_store::pool::connect_single(&url).map_err(|error| error.to_string())?;
    if lkjmc_store::bootstrap::try_apply_lock(&mut lock).map_err(|error| error.to_string())? {
        Ok(lock)
    } else {
        Err("another bootstrap apply is running".to_string())
    }
}

fn run_effects(
    state: &AppState,
    request: &CommandEnvelope,
    effects: Vec<BootstrapEffect>,
    diagnostics: Result<Value, serde_json::Error>,
    client: lkjmc_store::pool::PooledConnection,
) -> Result<Value, String> {
    let mut client = Some(client);
    (|| {
        let database = client.as_mut().ok_or("bootstrap connection unavailable")?;
        if effects
            .iter()
            .any(|effect| matches!(effect, BootstrapEffect::EnsureMigrations))
        {
            lkjmc_store::migrate::apply(database).map_err(|error| error.to_string())?;
        }
        lkjmc_store::bootstrap::fail_unfinished_runs(database)
            .map_err(|error| error.to_string())?;
        let run_id = Uuid::new_v4();
        create_run(database, run_id, request, diagnostics)?;
        for (index, effect) in effects.iter().enumerate() {
            if let BootstrapEffect::WaitForReadiness { id } = effect {
                if let Err(error) =
                    readiness_wait::run(state, &mut client, run_id, index, effect, id.as_str())
                {
                    if let Some(database) = client.as_mut() {
                        let _ = finish(database, run_id, "failed");
                    }
                    return Err(error);
                }
                continue;
            }
            let result = effects::apply_effect(
                state,
                request,
                client.as_mut().ok_or("bootstrap connection unavailable")?,
                effect,
            );
            let database = client.as_mut().ok_or("bootstrap connection unavailable")?;
            steps::record(database, run_id, index, effect, &result)?;
            if let Err(error) = result {
                finish(database, run_id, "failed")?;
                return Err(error);
            }
        }
        let database = client.as_mut().ok_or("bootstrap connection unavailable")?;
        lkjmc_store::shop::seed_default_catalog(database).map_err(|error| error.to_string())?;
        finish(database, run_id, "succeeded")?;
        super::status_body(state).map(|mut body| {
            body["result"] = json!("succeeded");
            body["runId"] = json!(run_id.to_string());
            body
        })
    })()
}

#[cfg(test)]
#[path = "apply_tests.rs"]
mod tests;

fn create_run(
    client: &mut postgres::Client,
    run_id: Uuid,
    request: &CommandEnvelope,
    diagnostics: Result<Value, serde_json::Error>,
) -> Result<(), String> {
    lkjmc_store::bootstrap::create_run(
        client,
        lkjmc_store::bootstrap::NewBootstrapRun {
            id: run_id,
            profile: "playable",
            requested_by: &request.actor.name,
            result: "running",
            diagnostics: diagnostics.unwrap_or_else(|_| json!([])),
        },
    )
    .map_err(|error| error.to_string())
}

fn finish(client: &mut postgres::Client, run_id: Uuid, result: &str) -> Result<(), String> {
    lkjmc_store::bootstrap::finish_run(client, run_id, result, json!([]))
        .map_err(|error| error.to_string())
}
