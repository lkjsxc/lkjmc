use lkjmc_core::command::CommandEnvelope;

use crate::api;
use crate::app::AppState;

pub fn handle(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    match request.command.as_str() {
        "instance.list" => crate::instance_read::list(state, request),
        "instance.logs" => crate::instance_read::logs(state, request),
        "instance.heartbeat" => crate::instance_heartbeat::handle(state, request),
        "instance.wake.request" => crate::instance_wake_join::request(state, request),
        "instance.create" => crate::instance_lifecycle::create(state, request),
        "instance.start" => crate::instance_lifecycle::start(state, request),
        "instance.stop" => crate::instance_lifecycle::stop(state, request),
        "instance.restart" => crate::instance_lifecycle::restart(state, request),
        "instance.delete" => crate::instance_lifecycle::delete(state, request),
        _ => api::error(
            request,
            "command.unknown",
            "unknown instance command",
            false,
        ),
    }
}
