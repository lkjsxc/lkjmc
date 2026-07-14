mod effects;
mod lock;
mod network_record;
mod readiness_wait;
mod steps;

use lkjmc_core::bootstrap::{BootstrapEffect, DiagnosticSeverity};
use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;
use crate::commands::adventure_confirmation;
use crate::dispatch as api;

pub fn apply(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    if !adventure_confirmation::accepted(&request.body) {
        return adventure_confirmation::required(request);
    }
    let bootstrap_request = match super::request::from_body(state, &request.body, false) {
        Ok(request) => request,
        Err(error) => return api::error(request, "bootstrap.request", error, false),
    };
    if let Err(error) = super::database_url(state) {
        return api::error(request, "bootstrap.apply_failed", error, false);
    }
    let guard = match lock::acquire(&state.data_root()) {
        Ok(value) => value,
        Err(error) => return api::error(request, "bootstrap.locked", error, true),
    };
    let inspection = match super::network_state::inspect(state) {
        Ok(value) => value,
        Err(error) => return api::error(request, "bootstrap.inspect_failed", error, false),
    };
    let facts = crate::commands::bootstrap_facts::gather(state);
    let plan = lkjmc_core::bootstrap::plan_bootstrap(&bootstrap_request, &facts);
    let admission = match network_record::admit(state, &request, &inspection) {
        Ok(value) => value,
        Err(error) => return api::error(request, "bootstrap.intent_failed", error, false),
    };
    match admission {
        network_record::Admission::Unsupported(id, reason) => api::error(
            request, "bootstrap.unsupported", format!("{reason}; attempt={id}"), false,
        ),
        network_record::Admission::NoOp(id) => match super::status_body(state, &request.body) {
            Ok(mut body) => {
                body["result"] = json!("no-op");
                body["networkAttemptId"] = json!(id.to_string());
                api::ok(request, body)
            }
            Err(error) => api::error(request, "bootstrap.status_failed", error, false),
        },
        network_record::Admission::Applying(id) => {
            if plan.diagnostics.iter().any(|item| item.severity == DiagnosticSeverity::Blocking) {
                let error = blocking_message(&plan);
                let _ = network_record::finish(state, id, "failed", Some(&error), json!({}));
                return api::error(request, "bootstrap.blocked", error, false);
            }
            if let Err(error) = guard.remaining() {
                let _ = network_record::finish(state, id, "failed", Some(&error), json!({}));
                return api::error(request, "bootstrap.deadline", error, false);
            }
            let result = run_plan(state, &request, plan.effects, serde_json::to_value(plan.diagnostics));
            match result {
                Ok(mut body) => {
                    let observed = super::network_state::inspect(state)
                        .and_then(|value| serde_json::to_value(value).map_err(|error| error.to_string()))
                        .unwrap_or_else(|error| json!({"observationError": error}));
                    if let Err(error) = network_record::finish(state, id, "observed", None, observed) {
                        return api::error(request, "bootstrap.observation_failed", error, false);
                    }
                    body["networkAttemptId"] = json!(id.to_string());
                    api::ok(request, body)
                }
                Err(error) => {
                    let _ = network_record::finish(state, id, "failed", Some(&error), json!({}));
                    api::error(request, "bootstrap.apply_failed", error, false)
                }
            }
        }
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
    state
        .database_connection()
        .and_then(|client| run_effects(state, request, effects, diagnostics, client))
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
            let result = if matches!(
                effect,
                BootstrapEffect::StartInstance { .. } | BootstrapEffect::RestartInstance { .. }
            ) {
                drop(client.take());
                let result = effects::apply_runtime_effect(state, effect);
                client = Some(state.database_connection()?);
                result
            } else {
                effects::apply_effect(
                    state,
                    request,
                    client.as_mut().ok_or("bootstrap connection unavailable")?,
                    effect,
                )
            };
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
        super::status_body(state, &request.body).map(|mut body| {
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
