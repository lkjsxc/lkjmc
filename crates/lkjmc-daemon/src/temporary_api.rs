mod cleanup;
mod create;
pub(crate) mod create_support;
pub(crate) mod lifecycle;
mod readiness;
pub(crate) mod request;
mod transfer;

use lkjmc_core::command::{CommandEnvelope, CommandResponse};

use crate::api;
use crate::app::AppState;

pub fn handle(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    match request.command.as_str() {
        "temporary.instance.create" => create::handle(state, request),
        "temporary.instance.start" => lifecycle::start(state, request),
        "temporary.instance.stop" => lifecycle::stop(state, request),
        "temporary.instance.cleanup" => cleanup::cleanup(state, request),
        "temporary.instance.get" => lifecycle::get(state, request),
        "temporary.transfer.intent" => transfer::intent(state, request),
        _ => api::error(
            request,
            "command.unknown",
            "unknown temporary command",
            false,
        ),
    }
}
