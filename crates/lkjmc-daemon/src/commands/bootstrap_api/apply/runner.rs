use lkjmc_core::bootstrap::BootstrapEffect;
use lkjmc_core::command::CommandEnvelope;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;

pub(super) fn run_plan(
    state: &AppState,
    request: &CommandEnvelope,
    attempt_id: Uuid,
    effects: Vec<BootstrapEffect>,
    diagnostics: Result<Value, serde_json::Error>,
    guard: &super::lock::ApplyGuard,
) -> Result<Value, String> {
    if super::super::database_url(state)?.is_none() {
        return Err("Database URL is not configured".to_string());
    }
    state.database_connection().and_then(|client| {
        run_effects(
            state,
            request,
            attempt_id,
            effects,
            diagnostics,
            client,
            guard,
        )
    })
}

fn run_effects(
    state: &AppState,
    request: &CommandEnvelope,
    attempt_id: Uuid,
    effects: Vec<BootstrapEffect>,
    diagnostics: Result<Value, serde_json::Error>,
    client: lkjmc_store::pool::PooledConnection,
    guard: &super::lock::ApplyGuard,
) -> Result<Value, String> {
    let mut client = Some(client);
    (|| {
        let run_id = Uuid::new_v4();
        {
            let database = client.as_mut().ok_or("bootstrap connection unavailable")?;
            if effects
                .iter()
                .any(|effect| matches!(effect, BootstrapEffect::EnsureMigrations))
            {
                lkjmc_store::migrate::apply(database).map_err(|error| error.to_string())?;
            }
            lkjmc_store::bootstrap::fail_unfinished_runs(database)
                .map_err(|error| error.to_string())?;
            create_run(database, run_id, request, diagnostics)?;
        }
        for (index, effect) in effects.iter().enumerate() {
            guard.remaining()?;
            if let BootstrapEffect::WaitForReadiness { id } = effect {
                super::network_record::mark_phase_with_client(
                    client.as_mut().ok_or("bootstrap connection unavailable")?,
                    attempt_id,
                    "observation",
                )?;
                if let Err(error) = super::readiness_wait::run(
                    state,
                    &mut client,
                    run_id,
                    index,
                    effect,
                    id.as_str(),
                ) {
                    if let Some(database) = client.as_mut() {
                        let _ = finish(database, run_id, "failed");
                    }
                    return Err(error);
                }
                continue;
            }
            let runtime_effect = matches!(
                effect,
                BootstrapEffect::StartInstance { .. }
                    | BootstrapEffect::StopInstance { .. }
                    | BootstrapEffect::RestartInstance { .. }
            );
            let phase = if runtime_effect {
                "runtime"
            } else {
                "configuration"
            };
            super::network_record::mark_phase_with_client(
                client.as_mut().ok_or("bootstrap connection unavailable")?,
                attempt_id,
                phase,
            )?;
            let result = if runtime_effect {
                drop(client.take());
                let result = super::effects::apply_runtime_effect(state, effect);
                client = Some(state.database_connection()?);
                result
            } else {
                super::effects::apply_effect(
                    state,
                    request,
                    client.as_mut().ok_or("bootstrap connection unavailable")?,
                    effect,
                )
            };
            let database = client.as_mut().ok_or("bootstrap connection unavailable")?;
            super::steps::record(database, run_id, index, effect, &result)?;
            if let Err(error) = result {
                finish(database, run_id, "failed")?;
                return Err(error);
            }
        }
        let database = client.as_mut().ok_or("bootstrap connection unavailable")?;
        lkjmc_store::shop::seed_default_catalog(database).map_err(|error| error.to_string())?;
        finish(database, run_id, "succeeded")?;
        drop(client.take());
        super::super::status_body(state, &request.body).map(|mut body| {
            body["result"] = json!("succeeded");
            body["runId"] = json!(run_id.to_string());
            body
        })
    })()
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
