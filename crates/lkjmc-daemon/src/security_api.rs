use lkjmc_core::command::{CommandEnvelope, CommandResponse};

use crate::api;
use crate::app::AppState;

pub fn handle(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    match request.command.as_str() {
        "security.daemon-token.plan" => crate::security_token::plan(state, request),
        "security.daemon-token.status" => crate::security_token::status(state, request),
        "security.daemon-token.rotate" => crate::security_token::rotate(state, request),
        "security.daemon-token.verify" => crate::security_token::verify(state, request),
        _ => api::error(
            request,
            "command.unknown",
            "unknown security command",
            false,
        ),
    }
}
