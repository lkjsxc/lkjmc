use serde_json::json;

use crate::commands::daemon_command;
use crate::error::CliError;

pub fn inspect(socket: &str, player_uuid: String, json_output: bool) -> Result<(), CliError> {
    daemon_command(
        socket,
        "player.inspect",
        json!({"playerUuid": player_uuid}),
        json_output,
        "ok player inspect",
    )
}

pub fn points_top(socket: &str, limit: i64, json_output: bool) -> Result<(), CliError> {
    daemon_command(
        socket,
        "player.points.top",
        json!({"limit": limit}),
        json_output,
        "ok player points top",
    )
}

pub fn restore(
    socket: &str,
    player_uuid: String,
    snapshot_id: String,
    json_output: bool,
) -> Result<(), CliError> {
    daemon_command(
        socket,
        "player.restore",
        json!({"playerUuid": player_uuid, "snapshotId": snapshot_id, "sourceInstance": "cli-restore"}),
        json_output,
        "ok player restore",
    )
}

pub fn snapshot(
    _socket: &str,
    _player_uuid: String,
    _name: String,
    _source: String,
    _payload_path: String,
    _json_output: bool,
) -> Result<(), CliError> {
    Err(CliError::message(
        "typed profile snapshot command contract is unavailable",
    ))
}
