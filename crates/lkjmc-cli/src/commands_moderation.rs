use serde_json::json;

use crate::args_moderation::ModerationCommand;
use crate::commands::daemon_command;
use crate::error::CliError;

pub fn run(socket: &str, command: ModerationCommand, json_output: bool) -> Result<(), CliError> {
    match command {
        ModerationCommand::Reports { limit } => daemon_command(
            socket,
            "player.report.list",
            json!({"limit": limit}),
            json_output,
            "ok moderation reports",
        ),
        ModerationCommand::ReportClose { report_id, status } => daemon_command(
            socket,
            &format!("player.report.{status}"),
            json!({"reportId": report_id}),
            json_output,
            "ok moderation report close",
        ),
        ModerationCommand::Warn {
            player_uuid,
            player_name,
            reason,
        } => daemon_command(
            socket,
            "player.warning.create",
            json!({
                "playerUuid": player_uuid,
                "playerName": player_name,
                "actorName": "cli",
                "reason": reason
            }),
            json_output,
            "ok moderation warn",
        ),
        ModerationCommand::Warnings { player_uuid, limit } => daemon_command(
            socket,
            "player.warning.list",
            json!({"playerUuid": player_uuid, "limit": limit}),
            json_output,
            "ok moderation warnings",
        ),
        ModerationCommand::Note {
            player_uuid,
            player_name,
            body,
        } => daemon_command(
            socket,
            "player.note.create",
            json!({
                "playerUuid": player_uuid,
                "playerName": player_name,
                "actorName": "cli",
                "body": body
            }),
            json_output,
            "ok moderation note",
        ),
        ModerationCommand::Notes { player_uuid, limit } => daemon_command(
            socket,
            "player.note.list",
            json!({"playerUuid": player_uuid, "limit": limit}),
            json_output,
            "ok moderation notes",
        ),
        ModerationCommand::Ban {
            player_uuid,
            player_name,
            reason,
        } => daemon_command(
            socket,
            "player.moderation.ban",
            json!({
                "playerUuid": player_uuid,
                "playerName": player_name,
                "actorName": "cli",
                "reason": reason
            }),
            json_output,
            "ok moderation ban",
        ),
        ModerationCommand::Unban { player_name } => daemon_command(
            socket,
            "player.moderation.unban",
            json!({"playerName": player_name}),
            json_output,
            "ok moderation unban",
        ),
        ModerationCommand::Mute {
            player_uuid,
            player_name,
            reason,
        } => daemon_command(
            socket,
            "player.moderation.mute",
            json!({
                "playerUuid": player_uuid,
                "playerName": player_name,
                "actorName": "cli",
                "reason": reason
            }),
            json_output,
            "ok moderation mute",
        ),
        ModerationCommand::Unmute { player_name } => daemon_command(
            socket,
            "player.moderation.unmute",
            json!({"playerName": player_name}),
            json_output,
            "ok moderation unmute",
        ),
        ModerationCommand::Status { player_uuid } => daemon_command(
            socket,
            "player.moderation.status",
            json!({"playerUuid": player_uuid}),
            json_output,
            "ok moderation status",
        ),
    }
}
