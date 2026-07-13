use lkjmc_core::command::CommandEnvelope;

use crate::app::AppState;
use crate::dispatch as api;

pub fn restore(
    _state: &AppState,
    request: CommandEnvelope,
) -> lkjmc_core::command::CommandResponse {
    api::error(
        request,
        "command.denied_unproved",
        "typed profile restore contract is unavailable",
        false,
    )
}
