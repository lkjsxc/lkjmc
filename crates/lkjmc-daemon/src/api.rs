use lkjmc_core::command::{CommandEnvelope, CommandErrorBody, CommandResponse};
use serde_json::{json, Value};

use crate::app::AppState;

pub fn dispatch(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let command_name = request.command.clone();
    match command_name.as_str() {
        "jar.prune" => crate::jar_prune::handle(state, request),
        "jar.sync" => crate::downloads::handle(state, request),
        "player.home.get" => crate::player_homes_api::get(state, request),
        "player.home.set" => crate::player_homes_api::set(state, request),
        "player.points.balance" => crate::player_points_api::balance(state, request),
        command if command.starts_with("player.") => crate::player_api::handle(state, request),
        command if command.starts_with("instance.") => crate::instance_api::handle(state, request),
        command if command.starts_with("jar.") => crate::jars::handle(state, request),
        "doctor" => ok(
            request,
            json!({
                "daemon": "ok",
                "databaseConfigured": state.database_url.is_some()
            }),
        ),
        "status" => ok(request, json!({"daemon": "running", "instances": []})),
        "audit.tail" => audit_tail(state, request),
        command => error(
            request,
            "command.unknown",
            format!("Unknown command: {command}"),
            false,
        ),
    }
}

fn audit_tail(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let limit = request
        .body
        .get("lines")
        .and_then(Value::as_i64)
        .unwrap_or(100)
        .clamp(1, 500);
    let Some(database_url) = &state.database_url else {
        return error(
            request,
            "database.not_configured",
            "Database URL is not configured",
            false,
        );
    };
    match lkjmc_store::pool::connect(database_url).and_then(|mut client| {
        lkjmc_store::audit::tail(&mut client, limit).map(|rows| {
            rows.into_iter()
                .map(|row| {
                    json!({
                        "actorKind": row.actor_kind,
                        "actorName": row.actor_name,
                        "action": row.action,
                        "targetKind": row.target_kind,
                        "targetId": row.target_id,
                        "result": row.result
                    })
                })
                .collect::<Vec<Value>>()
        })
    }) {
        Ok(events) => ok(request, json!({"events": events})),
        Err(error_value) => error(request, "database.error", error_value.to_string(), false),
    }
}

pub fn ok(request: CommandEnvelope, body: Value) -> CommandResponse {
    CommandResponse {
        request_id: request.request_id,
        ok: true,
        body: Some(body),
        error: None,
    }
}

pub fn error(
    request: CommandEnvelope,
    code: &str,
    message: impl Into<String>,
    retryable: bool,
) -> CommandResponse {
    CommandResponse {
        request_id: request.request_id,
        ok: false,
        body: None,
        error: Some(CommandErrorBody {
            code: code.to_string(),
            message: message.into(),
            retryable,
        }),
    }
}

#[cfg(test)]
mod tests {
    use lkjmc_core::command::{Actor, ActorKind};
    use lkjmc_core::id::CommandId;

    use super::*;

    #[test]
    fn status_reports_running() -> Result<(), lkjmc_core::error::IdError> {
        let request = CommandEnvelope {
            request_id: CommandId::parse("request id", "test")?,
            actor: Actor {
                kind: ActorKind::Cli,
                name: "test".to_string(),
            },
            command: "status".to_string(),
            body: json!({}),
        };
        let response = dispatch(
            &AppState::with_roots(
                None,
                "/tmp/lkjmc-test".to_string(),
                "/tmp/lkjmc-jars".to_string(),
                "/tmp/lkjmc-instances".to_string(),
            ),
            request,
        );
        assert!(response.ok);
        assert_eq!(
            response.body,
            Some(json!({"daemon": "running", "instances": []}))
        );
        Ok(())
    }
}
