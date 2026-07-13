use crate::app::AppState;
use crate::dispatch as api;

pub fn intent(
    _state: &AppState,
    envelope: lkjmc_core::command::CommandEnvelope,
) -> lkjmc_core::command::CommandResponse {
    api::error(
        envelope,
        "command.denied_unproved",
        "trusted transfer acknowledgement is unavailable",
        false,
    )
}
