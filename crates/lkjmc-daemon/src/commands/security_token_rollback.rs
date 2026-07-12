use lkjmc_core::command::{CommandEnvelope, CommandResponse};

use crate::app::AppState;
use crate::dispatch as api;

pub(super) fn rotation_failure<F>(
    state: &AppState,
    request: CommandEnvelope,
    path: &str,
    old: &str,
    new: &str,
    error: String,
    write: &mut F,
) -> CommandResponse
where
    F: FnMut(&str, &str) -> Result<(), String>,
{
    let rollback = rollback(state, path, old, write);
    super::write_audit(
        state,
        &request,
        Some(old),
        new,
        if rollback.is_ok() {
            "rolled-back"
        } else {
            "rollback-failed"
        },
    );
    api::error(
        request,
        "security.rotation_probe_failed",
        format!("{error}; rollback={}", rollback.is_ok()),
        true,
    )
}

fn rollback<F>(state: &AppState, path: &str, old: &str, write: &mut F) -> Result<(), String>
where
    F: FnMut(&str, &str) -> Result<(), String>,
{
    state.restore_web_bootstrap(old.to_string())?;
    match write(path, old) {
        Ok(()) => Ok(()),
        Err(error) => state.clear_web_bootstrap().and(Err(error)),
    }
}
