use std::fs;

use base64::Engine;
use serde_json::json;
use sha2::{Digest, Sha256};

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
    socket: &str,
    player_uuid: String,
    name: String,
    source: String,
    payload_path: String,
    json_output: bool,
) -> Result<(), CliError> {
    let payload = fs::read(payload_path)?;
    let sha256 = hex(&Sha256::digest(&payload));
    let payload_base64 = base64::engine::general_purpose::STANDARD.encode(payload);
    daemon_command(
        socket,
        "player.snapshot",
        json!({
            "playerUuid": player_uuid,
            "name": name,
            "sourceInstance": source,
            "scope": "profile",
            "payloadBase64": payload_base64,
            "sha256": sha256
        }),
        json_output,
        "ok player snapshot",
    )
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
