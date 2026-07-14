use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::audit_helpers::audit;
use crate::support::instance_helpers::{
    body_string, runtime_cancellation_state, stop_runtime, store,
};

pub(super) fn handle(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let result = (|| {
        let session_id = parse_uuid(&request, "sessionId")?;
        let reason = body_string(&request.body, "reason")?;
        let session = {
            let mut client = state.database_connection()?;
            store(lkjmc_store::temporary::get_session(
                &mut *client,
                session_id,
            ))?
            .ok_or_else(|| format!("adventure session not found: {session_id}"))?
        };
        if !cancellable(&session.state) {
            return Err(format!(
                "session cannot be cancelled from {}",
                session.state
            ));
        }
        let running = runtime_cancellation_state(state, &session.temporary_instance_id)?;
        if running {
            stop_runtime(state, &session.temporary_instance_id)?;
        }
        let mut client = state.database_connection()?;
        store(lkjmc_store::instance::update_desired_state(
            &mut client,
            &session.temporary_instance_id,
            "stopped",
        ))?;
        store(lkjmc_store::temporary::update_instance_state(
            &mut *client,
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
            &mut *client,
            &request,
            "adventure.session.cancel",
            "adventure-session",
            &session_id.to_string(),
            "succeeded",
        )?;
        Ok(api::ok(
            request.clone(),
            json!({
                "sessionId":session_id.to_string(),"cancelled":true,
                "stopped":true,"refundLedgerId":refund.to_string()
            }),
        ))
    })();
    result.unwrap_or_else(|error| api::error(request, "adventure.error", error, false))
}

#[cfg(test)]
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

    struct ChildGuard(std::process::Child);

    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    #[test]
    fn fenced_runtime_cancellation_does_not_persist_state_or_refund() -> Result<(), String> {
        let mut command = std::process::Command::new("sleep");
        command.arg("5").process_group(0);
        let mut child = ChildGuard(command.spawn().map_err(|error| error.to_string())?);
        let runtime = LocalRuntime::new();
        let instance_id = format!("fenced-{}", uuid::Uuid::new_v4().simple());
        let mut identity = process::identity(child.0.id())?;
        identity.start_ticks = identity.start_ticks.saturating_add(1);
        assert!(!runtime.recover(&instance_id, identity).healthy);
        let state = state().with_runtime(std::sync::Arc::new(runtime))?;
        let state_writes = Cell::new(0);
        let refunds = Cell::new(0);
        let result = {
            let result = after_verified_runtime(&state, &instance_id, |_| {
                state_writes.set(state_writes.get() + 1);
                refunds.set(refunds.get() + 1);
                Ok(())
            });
            assert!(result.is_err());
            assert_eq!(state_writes.get(), 0);
            assert_eq!(refunds.get(), 0);
            assert!(process::group_exists(child.0.id()));
            Ok(())
        };
        child.0.kill().map_err(|error| error.to_string())?;
        child.0.wait().map_err(|error| error.to_string())?;
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
