use uuid::Uuid;

use super::network_plan::NetworkEffect;
use super::{effects, steps};
use crate::app::AppState;

pub(super) fn run(
    state: &AppState,
    attempt_id: Uuid,
    run_id: Uuid,
    index: usize,
    effect: &NetworkEffect,
    id: &str,
) -> Result<(), String> {
    let (port, step_id) = {
        let mut connection = state.database_connection()?;
        let port = effects::readiness::server_port(&mut connection, id)?;
        let step_id = steps::start(&mut connection, run_id, index, effect)?;
        (port, step_id)
    };
    let probe_result = effects::readiness::wait_running(state, id, port);
    let terminal = state.database_connection().and_then(|mut connection| {
        super::network_record::verify_fence_with_client(&mut connection, attempt_id)?;
        steps::complete(&mut connection, step_id, &probe_result)
    });
    terminal_result(probe_result, terminal)
}

pub(super) fn terminal_result(
    probe_result: Result<(), String>,
    terminal: Result<(), String>,
) -> Result<(), String> {
    match (probe_result, terminal) {
        (Ok(()), Err(error)) => Err(format!("post-wait readiness bookkeeping failed: {error}")),
        (Err(error), _) => Err(error),
        (Ok(()), Ok(())) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::terminal_result;

    #[test]
    fn postwait_bookkeeping_fault_is_an_error() -> Result<(), String> {
        let Err(error) = terminal_result(Ok(()), Err("injected ledger fault".to_string())) else {
            return Err("post-wait bookkeeping fault unexpectedly succeeded".to_string());
        };
        assert_eq!(
            error,
            "post-wait readiness bookkeeping failed: injected ledger fault"
        );
        Ok(())
    }

    #[test]
    fn readiness_failure_survives_postwait_bookkeeping_fault() -> Result<(), String> {
        let Err(error) = terminal_result(
            Err("instance did not become ready".to_string()),
            Err("injected ledger fault".to_string()),
        ) else {
            return Err("readiness failure unexpectedly succeeded".to_string());
        };
        assert_eq!(error, "instance did not become ready");
        Ok(())
    }
}
