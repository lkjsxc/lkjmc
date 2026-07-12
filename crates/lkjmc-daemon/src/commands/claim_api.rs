use lkjmc_core::command::{CommandEnvelope, CommandResponse};

use crate::app::AppState;
use crate::dispatch as api;

pub fn handle(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    match request.command.as_str() {
        "claim.create" => crate::commands::claim_create::create(state, request),
        "claim.delete" => crate::commands::claim_create::delete(state, request),
        "claim.list" => crate::commands::claim_read::list(state, request),
        "claim.snapshot" => crate::commands::claim_read::snapshot(state, request),
        "claim.trust" => crate::commands::claim_trust::trust(state, request),
        "claim.untrust" => crate::commands::claim_trust::untrust(state, request),
        _ => api::error(request, "command.unknown", "unknown claim command", false),
    }
}

#[cfg(test)]
mod tests {
    use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
    use lkjmc_core::id::CommandId;
    use serde_json::json;

    use crate::app::AppState;

    #[test]
    fn claim_create_is_denied_before_database_work() -> Result<(), String> {
        let request = CommandEnvelope {
            request_id: CommandId::parse("request id", "claim.create")
                .map_err(|error| error.to_string())?,
            actor: Actor {
                kind: ActorKind::Cli,
                name: "claim-test".to_string(),
            },
            command: "claim.create".to_string(),
            body: json!({
                "ownerUuid":"00000000-0000-0000-0000-000000000301",
                "ownerName":"Owner", "name":"Base", "instanceId":"survival",
                "worldName":"world", "chunkX":1, "chunkZ":2
            }),
        };
        let state = AppState::with_config_path(
            None,
            8,
            "/tmp/config".to_string(),
            "/tmp/logs".to_string(),
            "/tmp/jars".to_string(),
            "/tmp/data".to_string(),
            None,
            None,
            None,
        );
        let response = crate::dispatch::dispatch(&state, request);
        assert_eq!(
            response.error.map(|error| error.code),
            Some("command.effect_denied".to_string())
        );
        Ok(())
    }
}
