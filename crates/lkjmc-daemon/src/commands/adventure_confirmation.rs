use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::Value;

use crate::dispatch as api;

pub(crate) const CODE: &str = "adventure.confirmation_required";
const MESSAGE: &str = "confirm Minecraft EULA acceptance before starting this adventure";

pub(crate) fn accepted(body: &Value) -> bool {
    body.get("acceptMinecraftEula").and_then(Value::as_bool) == Some(true)
}

pub(crate) fn required(request: CommandEnvelope) -> CommandResponse {
    api::error(request, CODE, MESSAGE, false)
}

#[cfg(test)]
mod tests {
    use lkjmc_core::command::{Actor, ActorKind};
    use lkjmc_core::id::CommandId;
    use serde_json::json;

    use super::*;

    #[test]
    fn absent_or_false_consent_has_one_non_retryable_contract() -> Result<(), String> {
        for body in [json!({}), json!({"acceptMinecraftEula": false})] {
            let response = required(request(body)?);
            assert!(!response.ok);
            assert!(response.body.is_none());
            assert_eq!(
                response.error.map(|error| (error.code, error.retryable)),
                Some((CODE.to_string(), false))
            );
        }
        Ok(())
    }

    fn request(body: Value) -> Result<CommandEnvelope, String> {
        Ok(CommandEnvelope {
            request_id: CommandId::parse("request id", "adventure.purchase")
                .map_err(|error| error.to_string())?,
            actor: Actor {
                kind: ActorKind::Cli,
                name: "consent-test".to_string(),
            },
            command: "adventure.purchase".to_string(),
            body,
        })
    }
}
