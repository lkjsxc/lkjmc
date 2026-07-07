use lkjmc_core::command::{CommandEnvelope, CommandResponse};

use crate::app::AppState;
use crate::dispatch as api;

pub fn handle(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    match request.command.as_str() {
        "security.daemon-token.plan" => crate::commands::security_token::plan(state, request),
        "security.daemon-token.status" => crate::commands::security_token::status(state, request),
        "security.daemon-token.rotate" => crate::commands::security_token::rotate(state, request),
        "security.daemon-token.verify" => crate::commands::security_token::verify(state, request),
        "security.daemon-token.create" => {
            crate::commands::security_scoped_token::create(state, request)
        }
        "security.daemon-token.revoke" => {
            crate::commands::security_scoped_token::revoke(state, request)
        }
        _ => api::error(
            request,
            "command.unknown",
            "unknown security command",
            false,
        ),
    }
}
