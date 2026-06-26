use lkjmc_core::command::{CommandEnvelope, CommandResponse};

use crate::api;
use crate::app::AppState;

pub fn handle(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    match request.command.as_str() {
        "claim.create" => crate::claim_create::create(state, request),
        "claim.delete" => crate::claim_create::delete(state, request),
        "claim.list" => crate::claim_read::list(state, request),
        "claim.snapshot" => crate::claim_read::snapshot(state, request),
        "claim.trust" => crate::claim_trust::trust(state, request),
        "claim.untrust" => crate::claim_trust::untrust(state, request),
        _ => api::error(request, "command.unknown", "unknown claim command", false),
    }
}
