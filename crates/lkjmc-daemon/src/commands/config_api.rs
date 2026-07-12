use lkjmc_core::command::CommandEnvelope;

use crate::app::AppState;
use crate::dispatch as api;

type Response = lkjmc_core::command::CommandResponse;

pub fn reload(_state: &AppState, request: CommandEnvelope) -> Response {
    api::error(
        request,
        "config.restart_required",
        "configuration applies only at daemon restart; no config was read or applied",
        false,
    )
}
