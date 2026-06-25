use lkjmc_core::command::{CommandEnvelope, CommandErrorBody, CommandResponse};
use serde_json::{json, Value};

use crate::app::AppState;

pub fn dispatch(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let command_name = request.command.clone();
    match command_name.as_str() {
        "jar.prune" => crate::jar_prune::handle(state, request),
        "jar.sync" => crate::downloads::handle(state, request),
        "player.achievement.grant" => crate::player_achievements_api::grant(state, request),
        "player.achievements.list" => crate::player_achievements_api::list(state, request),
        "player.home.get" => crate::player_homes_api::get(state, request),
        "player.home.set" => crate::player_homes_api::set(state, request),
        "player.party.accept" => crate::player_party_api::accept(state, request),
        "player.party.create" => crate::player_party_api::create(state, request),
        "player.party.info" => crate::player_party_api::info(state, request),
        "player.party.invite" => crate::player_party_api::invite(state, request),
        "player.party.leave" => crate::player_party_api::leave(state, request),
        "player.points.balance" => crate::player_points_api::balance(state, request),
        "player.settings.get" => crate::player_settings_api::get(state, request),
        "player.settings.hud" => crate::player_settings_api::set_hud(state, request),
        "player.session.join" => crate::player_session_api::join(state, request),
        "player.session.leave" => crate::player_session_api::leave(state, request),
        "player.settings.set" => crate::player_settings_api::set_language(state, request),
        "player.shop.list" => crate::player_shop_api::list(state, request),
        "player.shop.purchase" => crate::player_shop_api::purchase(state, request),
        "player.teleport.request" => crate::player_teleport_api::request(state, request),
        "player.teleport.take" => crate::player_teleport_api::take(state, request),
        "shop.item.upsert" => crate::player_shop_api::upsert_item(state, request),
        "player.warp.get" => crate::player_warps_api::get(state, request),
        "player.warp.set" => crate::player_warps_api::set(state, request),
        "config.reload" => crate::config_api::reload(state, request),
        command if command.starts_with("player.") => crate::player_api::handle(state, request),
        command if command.starts_with("instance.") => crate::instance_api::handle(state, request),
        command if command.starts_with("jar.") => crate::jars::handle(state, request),
        "doctor" => ok(
            request,
            json!({
                "daemon": "ok",
                "databaseConfigured": state.database_url().is_some()
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
    let Some(database_url) = state.database_url() else {
        return error(
            request,
            "database.not_configured",
            "Database URL is not configured",
            false,
        );
    };
    match lkjmc_store::pool::connect(&database_url).and_then(|mut client| {
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
            &AppState::with_config_path(
                None,
                "/tmp/lkjmc-config".to_string(),
                "/tmp/lkjmc-test".to_string(),
                "/tmp/lkjmc-jars".to_string(),
                "/tmp/lkjmc-instances".to_string(),
                None,
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
