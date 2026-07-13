use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::audit_helpers::audit;
use crate::support::instance_helpers::{
    body_string, runtime_cancellation_state, stop_runtime, store, with_connection,
};

pub(super) fn handle(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    with_connection(state, request, |state, request, client| {
        let session_id = parse_uuid(&request, "sessionId")?;
        let reason = body_string(&request.body, "reason")?;
        let session = store(lkjmc_store::temporary::get_session(client, session_id))?
            .ok_or_else(|| format!("adventure session not found: {session_id}"))?;
        if !cancellable(&session.state) {
            return Err(format!(
                "session cannot be cancelled from {}",
                session.state
            ));
        }
        after_verified_runtime(state, &session.temporary_instance_id, |running| {
            if running {
                stop_runtime(state, client, &session.temporary_instance_id)?;
            }
            store(lkjmc_store::instance::update_desired_state(
                client,
                &session.temporary_instance_id,
                "stopped",
            ))?;
            store(lkjmc_store::temporary::update_instance_state(
                client,
                &session.temporary_instance_id,
                "stopped",
                None,
            ))?;
            let mut transaction = client.transaction().map_err(|error| error.to_string())?;
            let refund = store(lkjmc_store::temporary::refund_session(
                &mut transaction,
                session_id,
                "adventure-cancel-refund",
                &reason,
            ))?
            .ok_or_else(|| "cancelled session is not eligible for refund".to_string())?;
            store(lkjmc_store::temporary::update_session_state(
                &mut transaction,
                session_id,
                "cancelled",
                Some(&reason),
                Some(refund),
            ))?;
            transaction.commit().map_err(|error| error.to_string())?;
            audit(
                client,
                &request,
                "adventure.session.cancel",
                "adventure-session",
                &session_id.to_string(),
                "succeeded",
            )?;
            Ok(api::ok(
                request,
                json!({
                    "sessionId": session_id.to_string(),
                    "cancelled": true,
                    "stopped": true,
                    "refundLedgerId": refund.to_string()
                }),
            ))
        })
    })
}

fn after_verified_runtime<T>(
    state: &AppState,
    instance_id: &str,
    persist: impl FnOnce(bool) -> Result<T, String>,
) -> Result<T, String> {
    persist(runtime_cancellation_state(state, instance_id)?)
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}

fn cancellable(state: &str) -> bool {
    matches!(state, "pending" | "starting" | "ready")
}

#[cfg(test)]
#[path = "cancel_tests.rs"]
mod tests;

#[cfg(test)]
mod local_tests {
    use std::cell::Cell;
    use std::os::unix::process::CommandExt;

    use super::{after_verified_runtime, cancellable};
    use crate::app::AppState;
    use crate::runtime::local::LocalRuntime;
    use crate::runtime::process;

    #[test]
    fn fenced_runtime_cancellation_does_not_persist_state_or_refund() -> Result<(), String> {
        let mut command = std::process::Command::new("sleep");
        command.arg("5").process_group(0);
        let mut child = command.spawn().map_err(|error| error.to_string())?;
        let runtime = LocalRuntime::new();
        let mut identity = process::identity(child.id())?;
        identity.start_ticks = identity.start_ticks.saturating_add(1);
        assert!(!runtime.recover("fenced", identity).healthy);
        let state = state().with_runtime(std::sync::Arc::new(runtime))?;
        let state_writes = Cell::new(0);
        let refunds = Cell::new(0);
        let result = {
            let result = after_verified_runtime(&state, "fenced", |_| {
                state_writes.set(state_writes.get() + 1);
                refunds.set(refunds.get() + 1);
                Ok(())
            });
            assert!(result.is_err());
            assert_eq!(state_writes.get(), 0);
            assert_eq!(refunds.get(), 0);
            assert!(process::group_exists(child.id()));
            Ok(())
        };
        child.kill().map_err(|error| error.to_string())?;
        child.wait().map_err(|error| error.to_string())?;
        result
    }

    #[test]
    fn cancellation_only_allows_pre_active_sessions() {
        assert!(cancellable("ready"));
        assert!(!cancellable("active"));
    }

    fn state() -> AppState {
        AppState::with_config_path(
            None,
            1,
            "/tmp/config".to_string(),
            "/tmp/log".to_string(),
            "/tmp/jars".to_string(),
            "/tmp/data".to_string(),
            None,
            None,
            None,
        )
    }
}
