use lkjmc_core::command::{CommandEnvelope, CommandErrorBody, CommandResponse};
use serde_json::{json, Value};

use crate::app::AppState;

pub fn dispatch(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let command_name = request.command.clone();
    match command_name.as_str() {
        "announcement.create" => crate::announcement_api::create(state, request),
        "announcement.recent" => crate::announcement_api::recent(state, request),
        "jar.prune" => crate::jar_prune::handle(state, request),
        "jar.sync" => crate::downloads::handle(state, request),
        "player.achievement.grant" => crate::player_achievements_api::grant(state, request),
        "player.achievements.list" => crate::player_achievements_api::list(state, request),
        "player.daily.claim" => crate::player_daily_api::claim(state, request),
        "player.home.get" => crate::player_homes_api::get(state, request),
        "player.home.set" => crate::player_homes_api::set(state, request),
        "player.kit.claim" => crate::player_kit_api::claim(state, request),
        "player.kit.list" => crate::player_kit_api::list(state, request),
        "player.mail.inbox" => crate::player_mail_api::inbox(state, request),
        "player.mail.read" => crate::player_mail_api::read(state, request),
        "player.mail.send" => crate::player_mail_api::send(state, request),
        "player.moderation.ban" => crate::player_moderation_api::ban(state, request),
        "player.moderation.mute" => crate::player_moderation_api::mute(state, request),
        "player.moderation.status" => crate::player_moderation_api::status(state, request),
        "player.moderation.unmute" => crate::player_moderation_api::unmute(state, request),
        "player.note.create" => crate::player_note_api::create(state, request),
        "player.note.list" => crate::player_note_api::list(state, request),
        "player.moderation.unban" => crate::player_moderation_api::unban(state, request),
        "player.party.accept" => crate::player_party_api::accept(state, request),
        "player.party.create" => crate::player_party_api::create(state, request),
        "player.party.info" => crate::player_party_api::info(state, request),
        "player.party.invite" => crate::player_party_api::invite(state, request),
        "player.party.leave" => crate::player_party_api::leave(state, request),
        "player.points.balance" => crate::player_points_api::balance(state, request),
        "player.points.top" => crate::player_points_api::top(state, request),
        "player.report.create" => crate::player_report_api::create(state, request),
        "player.report.dismiss" => crate::player_report_api::dismiss(state, request),
        "player.report.list" => crate::player_report_api::list(state, request),
        "player.report.resolve" => crate::player_report_api::resolve(state, request),
        "player.restore" => crate::player_restore_api::restore(state, request),
        "player.settings.get" => crate::player_settings_api::get(state, request),
        "player.settings.hud" => crate::player_settings_api::set_hud(state, request),
        "player.session.join" => crate::player_session_api::join(state, request),
        "player.session.leave" => crate::player_session_api::leave(state, request),
        "player.settings.set" => crate::player_settings_api::set_language(state, request),
        "player.shop.list" => crate::player_shop_api::list(state, request),
        "player.shop.purchase" => crate::player_shop_api::purchase(state, request),
        "player.teleport.request" => crate::player_teleport_api::request(state, request),
        "player.teleport.take" => crate::player_teleport_api::take(state, request),
        "player.vote.list" => crate::player_vote_api::list(state, request),
        "kit.upsert" => crate::player_kit_api::upsert(state, request),
        "vote.link.upsert" => crate::player_vote_api::upsert(state, request),
        "vote.reward" => crate::player_vote_api::reward(state, request),
        "player.warning.create" => crate::player_warning_api::create(state, request),
        "player.warning.list" => crate::player_warning_api::list(state, request),
        "shop.item.upsert" => crate::player_shop_api::upsert_item(state, request),
        "player.warp.get" => crate::player_warps_api::get(state, request),
        "player.warp.set" => crate::player_warps_api::set(state, request),
        "config.reload" => crate::config_api::reload(state, request),
        command if command.starts_with("player.") => crate::player_api::handle(state, request),
        command if command.starts_with("instance.") => crate::instance_api::handle(state, request),
        command if command.starts_with("jar.") => crate::jars::handle(state, request),
        "doctor" => crate::doctor_api::doctor(state, request),
        "status" => crate::status_api::status(state, request),
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
    fn status_reports_running() -> Result<(), String> {
        let request = CommandEnvelope {
            request_id: CommandId::parse("request id", "test")
                .map_err(|error| error.to_string())?,
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
        let body = response
            .body
            .ok_or_else(|| "status body missing".to_string())?;
        assert_eq!(body["daemon"], json!("running"));
        assert_eq!(body["database"]["configured"], json!(false));
        Ok(())
    }
}
