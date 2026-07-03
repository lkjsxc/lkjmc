mod effects;
mod steps;

use lkjmc_core::bootstrap::{BootstrapEffect, DiagnosticSeverity};
use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api;
use crate::app::AppState;

pub fn apply(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let bootstrap_request = match super::request::from_body(state, &request.body, false) {
        Ok(request) => request,
        Err(error) => return api::error(request, "bootstrap.request", error, false),
    };
    if !bootstrap_request.accept_minecraft_eula {
        return api::error(
            request,
            "bootstrap.eula_required",
            "pass --accept-minecraft-eula or set LKJMC_ACCEPT_MINECRAFT_EULA=1",
            false,
        );
    }
    let facts = crate::bootstrap_facts::gather(state);
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
    if state.database_url().is_none() {
        return Err("Database URL is not configured".to_string());
    }
    let mut client = state.database_connection()?;
    if effects
        .iter()
        .any(|effect| matches!(effect, BootstrapEffect::EnsureMigrations))
    {
        lkjmc_store::migrate::apply(&mut client).map_err(|error| error.to_string())?;
    }
    let run_id = Uuid::new_v4();
    create_run(&mut client, run_id, request, diagnostics)?;
    for (index, effect) in effects.iter().enumerate() {
        let result = effects::apply_effect(state, request, &mut client, effect);
        steps::record(&mut client, run_id, index, effect, &result)?;
        if let Err(error) = result {
            finish(&mut client, run_id, "failed")?;
            return Err(error);
        }
    }
    finish(&mut client, run_id, "succeeded")?;
    super::status_body(state).map(|mut body| {
        body["result"] = json!("succeeded");
        body["runId"] = json!(run_id.to_string());
        body
    })
}

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
