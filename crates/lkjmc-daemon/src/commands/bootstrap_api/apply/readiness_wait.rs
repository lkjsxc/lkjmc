use lkjmc_core::bootstrap::BootstrapEffect;
use uuid::Uuid;

use crate::app::AppState;

use super::{effects, steps};

type PrepareError = Box<(lkjmc_store::pool::PooledConnection, String)>;

pub(super) fn run(
    state: &AppState,
    client: &mut Option<lkjmc_store::pool::PooledConnection>,
    run_id: Uuid,
    index: usize,
    effect: &BootstrapEffect,
    id: &str,
) -> Result<(), String> {
    let port = effects::readiness::server_port(
        client.as_mut().ok_or("bootstrap connection unavailable")?,
        id,
    )?;
    let connection = client.take().ok_or("bootstrap connection unavailable")?;
    let (step_id, probe_result) = match record_then_release(
        connection,
        |database| steps::start(database, run_id, index, effect),
        || effects::readiness::wait_running(state, id, port),
    ) {
        Ok(value) => value,
        Err(error) => {
            let (connection, error) = *error;
            *client = Some(connection);
            return Err(error);
        }
    };
    reconnect_and_complete(state, client, step_id, probe_result, steps::complete)
}

pub(super) fn record_then_release<T>(
    mut connection: lkjmc_store::pool::PooledConnection,
    record: impl FnOnce(&mut postgres::Client) -> Result<T, String>,
    wait: impl FnOnce() -> Result<(), String>,
) -> Result<(T, Result<(), String>), PrepareError> {
    let recorded = match record(&mut connection) {
        Ok(value) => value,
        Err(error) => return Err(Box::new((connection, error))),
    };
    drop(connection);
    Ok((recorded, wait()))
}

pub(super) fn reconnect_and_complete(
    state: &AppState,
    client: &mut Option<lkjmc_store::pool::PooledConnection>,
    step_id: Uuid,
    probe_result: Result<(), String>,
    complete: impl FnOnce(&mut postgres::Client, Uuid, &Result<(), String>) -> Result<(), String>,
) -> Result<(), String> {
    let connection = match state.database_connection() {
        Ok(connection) => connection,
        Err(error) => return terminal_result(probe_result, Err(error)),
    };
    *client = Some(connection);
    let terminal = client
        .as_mut()
        .ok_or_else(|| "bootstrap connection unavailable".to_string())
        .and_then(|database| complete(database, step_id, &probe_result));
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
