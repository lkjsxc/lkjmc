use lkjmc_core::command::CommandEnvelope;
use serde_json::{json, Value};
use uuid::Uuid;

use super::network_plan::NetworkEffect;
use crate::app::AppState;

pub(super) fn run_plan(
    state: &AppState,
    request: &CommandEnvelope,
    attempt_id: Uuid,
    effects: Vec<NetworkEffect>,
    diagnostics: Result<Value, serde_json::Error>,
    guard: &super::lock::ApplyGuard,
) -> Result<Value, String> {
    if super::super::database_url(state)?.is_none() {
        return Err("Database URL is not configured".to_string());
    }
    let run_id = Uuid::new_v4();
    {
        let mut database = state.database_connection()?;
        lkjmc_store::bootstrap::fail_unfinished_runs(&mut database)
            .map_err(|error| error.to_string())?;
        create_run(&mut database, run_id, request, diagnostics)?;
    }
    for (index, effect) in effects.iter().enumerate() {
        guard.remaining()?;
        if let NetworkEffect::WaitForReadiness { id } = effect {
            super::network_record::mark_phase(state, attempt_id, "observation")?;
            if let Err(error) =
                super::readiness_wait::run(state, attempt_id, run_id, index, effect, id.as_str())
            {
                finish_state(state, run_id, "failed")?;
                return Err(error);
            }
            continue;
        }
        let runtime_effect = matches!(
            effect,
            NetworkEffect::StartInstance { .. } | NetworkEffect::StopInstance { .. }
        );
        super::network_record::mark_phase(
            state,
            attempt_id,
            if runtime_effect {
                "runtime"
            } else {
                "configuration"
            },
        )?;
        let result = if runtime_effect {
            super::effects::apply_runtime_effect(state, effect)
        } else {
            super::effects::apply_effect(state, request, effect)
        };
        {
            let mut database = state.database_connection()?;
            super::network_record::verify_fence_with_client(&mut database, attempt_id)?;
            super::steps::record(&mut database, run_id, index, effect, &result)?;
            if result.is_err() {
                finish(&mut database, run_id, "failed")?;
            }
        }
        result?;
    }
    {
        let mut database = state.database_connection()?;
        lkjmc_store::shop::seed_default_catalog(&mut database)
            .map_err(|error| error.to_string())?;
        finish(&mut database, run_id, "succeeded")?;
    }
    super::super::status_body(state, &request.body).map(|mut body| {
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

fn finish_state(state: &AppState, run_id: Uuid, result: &str) -> Result<(), String> {
    let mut client = state.database_connection()?;
    finish(&mut client, run_id, result)
}

fn finish(client: &mut postgres::Client, run_id: Uuid, result: &str) -> Result<(), String> {
    lkjmc_store::bootstrap::finish_run(client, run_id, result, json!([]))
        .map_err(|error| error.to_string())
}
