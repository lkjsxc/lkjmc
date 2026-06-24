use base64::Engine;
use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, store, with_client};

pub fn handle(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    match request.command.as_str() {
        "player.inspect" => inspect(state, request),
        "player.load" => load(state, request),
        "player.snapshot" => snapshot(state, request),
        _ => api::error(request, "command.unknown", "unknown player command", false),
    }
}

fn inspect(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    with_client(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request.body, "playerUuid")?;
        let scope = request
            .body
            .get("scope")
            .and_then(|v| v.as_str())
            .unwrap_or("profile");
        let name = store(lkjmc_store::player::get_identity_name(client, player_uuid))?;
        let count = store(lkjmc_store::player::snapshot_count(client, player_uuid))?;
        let latest = store(lkjmc_store::player::latest_snapshot(
            client,
            player_uuid,
            scope,
        ))?;
        Ok(api::ok(
            request,
            json!({
                "playerUuid": player_uuid.to_string(),
                "name": name,
                "snapshotCount": count,
                "latestRevision": latest.as_ref().map(|snapshot| snapshot.revision)
            }),
        ))
    })
}

fn load(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    with_client(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request.body, "playerUuid")?;
        let scope = request
            .body
            .get("scope")
            .and_then(|v| v.as_str())
            .unwrap_or("profile");
        let Some(snapshot) = store(lkjmc_store::player::latest_snapshot(
            client,
            player_uuid,
            scope,
        ))?
        else {
            return Ok(api::ok(request, json!({"found": false})));
        };
        let payload = base64::engine::general_purpose::STANDARD.encode(&snapshot.payload);
        Ok(api::ok(
            request,
            json!({
                "found": true,
                "playerUuid": snapshot.player_uuid.to_string(),
                "revision": snapshot.revision,
                "sha256": snapshot.sha256,
                "payloadBase64": payload
            }),
        ))
    })
}

fn snapshot(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    with_client(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request.body, "playerUuid")?;
        let name = body_string(&request.body, "name")?;
        let source = body_string(&request.body, "sourceInstance")?;
        let payload = decode_payload(&request.body)?;
        let scope = request
            .body
            .get("scope")
            .and_then(|v| v.as_str())
            .unwrap_or("profile");
        store(lkjmc_store::player::insert_identity(
            client,
            player_uuid,
            &name,
        ))?;
        let revision = store(lkjmc_store::player::acquire_lease(
            client,
            player_uuid,
            scope,
            &source,
        ))? + 1;
        let sha256 = body_string(&request.body, "sha256")?;
        store(lkjmc_store::player::insert_snapshot_with_metadata(
            client,
            lkjmc_store::player::NewSnapshot {
                id: Uuid::new_v4(),
                player_uuid,
                scope,
                revision,
                payload_format: "paper-bukkit-object-stream-v1",
                payload: &payload,
                sha256: &sha256,
                source_instance: &source,
                metadata: json!({}),
            },
        ))?;
        store(lkjmc_store::player::upsert_lease(
            client,
            player_uuid,
            scope,
            &source,
            revision,
        ))?;
        Ok(api::ok(
            request,
            json!({"playerUuid": player_uuid.to_string(), "revision": revision}),
        ))
    })
}

fn parse_uuid(body: &serde_json::Value, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(body, field)?).map_err(|error| error.to_string())
}

fn decode_payload(body: &serde_json::Value) -> Result<Vec<u8>, String> {
    base64::engine::general_purpose::STANDARD
        .decode(body_string(body, "payloadBase64")?)
        .map_err(|error| error.to_string())
}
