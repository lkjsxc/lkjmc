use lkjmc_core::command::CommandEnvelope;

use crate::app::AppState;
use crate::commands::adventure_confirmation;
use crate::dispatch as api;

pub fn handle(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    if requires_eula_confirmation(&request) {
        return adventure_confirmation::required(request);
    }
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

fn requires_eula_confirmation(request: &CommandEnvelope) -> bool {
    matches!(
        request.command.as_str(),
        "instance.create.plan" | "instance.create"
    ) && request
        .body
        .get("kind")
        .and_then(serde_json::Value::as_str)
        .is_some_and(lkjmc_core::instance_create::requires_eula)
        && !adventure_confirmation::accepted(&request.body)
}

#[cfg(test)]
mod tests {
    use lkjmc_core::command::{Actor, ActorKind};
    use lkjmc_core::id::CommandId;
    use serde_json::json;

    use super::*;

    #[test]
    fn minecraft_create_paths_require_the_shared_confirmation() -> Result<(), String> {
        for command in ["instance.create.plan", "instance.create"] {
            for body in [
                json!({"kind":"paper"}),
                json!({"kind":"paper","acceptMinecraftEula":false}),
            ] {
                let response = handle(&state(), request(command, body)?);
                assert!(!response.ok);
                assert!(response.body.is_none());
                assert_eq!(
                    response.error.map(|error| (error.code, error.retryable)),
                    Some((adventure_confirmation::CODE.to_string(), false))
                );
            }
        }
        Ok(())
    }

    fn request(command: &str, body: serde_json::Value) -> Result<CommandEnvelope, String> {
        Ok(CommandEnvelope {
            request_id: CommandId::parse("request id", command)
                .map_err(|error| error.to_string())?,
            actor: Actor {
                kind: ActorKind::Cli,
                name: "consent-test".to_string(),
            },
            command: command.to_string(),
            body,
        })
    }

    fn state() -> AppState {
        AppState::with_config_path(
            None,
            1,
            "/tmp/config".to_string(),
            "/tmp/logs".to_string(),
            "/tmp/jars".to_string(),
            "/tmp/data".to_string(),
            None,
            None,
            None,
        )
    }
}
