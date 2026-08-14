mod effects;
mod lock;
mod network_plan;
mod network_record;
mod network_recovery;
mod readiness_wait;
mod runner;
mod steps;

use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::json;

use crate::app::AppState;
use crate::commands::adventure_confirmation;
use crate::dispatch as api;

pub fn apply(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    if !adventure_confirmation::accepted(&request.body) {
        return adventure_confirmation::required(request);
    }
    if let Err(error) = super::request::validate(state, &request.body) {
        return api::error(request, "bootstrap.request", error, false);
    }
    if let Err(error) = super::database_url(state) {
        return api::error(request, "bootstrap.apply_failed", error, false);
    }
    let guard = match lock::acquire(&state.data_root()) {
        Ok(value) => value,
        Err(error) => return api::error(request, "bootstrap.locked", error, true),
    };
    if let Err(error) = network_recovery::recover(state) {
        return api::error(request, "bootstrap.recovery_unknown", error, true);
    }
    let inspection = match super::network_state::inspect(state) {
        Ok(value) => value,
        Err(error) => return api::error(request, "bootstrap.inspect_failed", error, false),
    };
    let admission = match network_record::admit(state, &request, &inspection) {
        Ok(value) => value,
        Err(error) => return api::error(request, "bootstrap.intent_failed", error, false),
    };
    match admission {
        network_record::Admission::Unsupported(id, reason) => api::error(
            request,
            "bootstrap.unsupported",
            format!("{reason}; attempt={id}"),
            false,
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
            let prepared = state
                .runtime_config()
                .and_then(|value| value.ok_or("runtime config is unavailable".to_string()))
                .and_then(|config| {
                    network_plan::register_assets(state, &config)?;
                    network_plan::effects(
                        &config,
                        &inspection,
                        adventure_confirmation::accepted(&request.body),
                    )
                });
            let effects = match prepared {
                Ok(value) => value,
                Err(error) => {
                    let _ = network_record::finish_error(state, id, &error);
                    return api::error(request, "bootstrap.prepare_failed", error, false);
                }
            };
            let result = runner::run_plan(
                state,
                &request,
                id,
                effects,
                serde_json::to_value(&inspection.changes),
                &guard,
            );
            match result {
                Ok(mut body) => {
                    if let Err(error) = network_record::mark_phase(state, id, "observation") {
                        return api::error(request, "bootstrap.observation_failed", error, false);
                    }
                    let observed = match super::network_state::inspect(state) {
                        Ok(value)
                            if value.outcome
                                == lkjmc_core::network_intent::InspectionOutcome::NoOp =>
                        {
                            match serde_json::to_value(value) {
                                Ok(value) => value,
                                Err(error) => {
                                    let error = error.to_string();
                                    let _ = network_record::finish_error(state, id, &error);
                                    return api::error(
                                        request,
                                        "bootstrap.observation_failed",
                                        error,
                                        false,
                                    );
                                }
                            }
                        }
                        Ok(value) => {
                            let error =
                                format!("post-apply network observation was {:?}", value.outcome);
                            let _ = network_record::finish_error(state, id, &error);
                            return api::error(
                                request,
                                "bootstrap.observation_failed",
                                error,
                                false,
                            );
                        }
                        Err(error) => {
                            let _ = network_record::finish_error(state, id, &error);
                            return api::error(
                                request,
                                "bootstrap.observation_failed",
                                error,
                                false,
                            );
                        }
                    };
                    if let Err(error) =
                        network_record::finish(state, id, "observed", None, observed)
                    {
                        return api::error(request, "bootstrap.observation_failed", error, false);
                    }
                    body["networkAttemptId"] = json!(id.to_string());
                    api::ok(request, body)
                }
                Err(error) => {
                    let _ = network_record::finish_error(state, id, &error);
                    api::error(request, "bootstrap.apply_failed", error, false)
                }
            }
        }
    }
}

#[cfg(test)]
mod network_probe_tests;
#[cfg(test)]
#[path = "apply_tests.rs"]
mod tests;
