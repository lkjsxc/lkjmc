use lkjmc_core::command::CommandEnvelope;

use crate::app::AppState;
use crate::dispatch as api;

pub fn handle(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    match request.command.as_str() {
        "instance.list" => crate::commands::instance_read::list(state, request),
        "instance.logs" => crate::commands::instance_read::logs(state, request),
        command if command.starts_with("instance.wake.") => {
            crate::commands::instance_wake_join::handle(state, request)
        }
        "instance.create.plan" => crate::commands::instance_create::plan(state, request),
        "instance.create" => crate::commands::instance_lifecycle::create(state, request),
        "instance.start" => crate::commands::instance_lifecycle::start(state, request),
        "instance.stop" => crate::commands::instance_lifecycle::stop(state, request),
        "instance.restart" => crate::commands::instance_lifecycle::restart(state, request),
        "instance.delete" => crate::commands::instance_lifecycle::delete(state, request),
        _ => api::error(
            request,
            "command.unknown",
            "unknown instance command",
            false,
        ),
    }
}
