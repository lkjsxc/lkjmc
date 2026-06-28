mod purchase;
mod rows;

use lkjmc_core::command::{CommandEnvelope, CommandResponse};

use crate::api;
use crate::app::AppState;

pub fn handle(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    match request.command.as_str() {
        "adventure.end.purchase" => purchase::end(state, request),
        _ => api::error(
            request,
            "command.unknown",
            "unknown adventure command",
            false,
        ),
    }
}
