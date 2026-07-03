use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, store, with_connection};

type Response = lkjmc_core::command::CommandResponse;

pub fn restore(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request.body, "playerUuid")?;
        let snapshot_id = parse_uuid(&request.body, "snapshotId")?;
        let source = request
            .body
            .get("sourceInstance")
            .and_then(|v| v.as_str())
            .unwrap_or("cli-restore");
        let scope = request
            .body
            .get("scope")
            .and_then(|v| v.as_str())
            .unwrap_or("profile");
        let snapshot = store(lkjmc_store::player::snapshot_by_id(
            client,
            snapshot_id,
            player_uuid,
            scope,
        ))?
        .ok_or_else(|| "snapshot not found for player".to_string())?;
        let revision = store(lkjmc_store::player::acquire_lease(
            client,
            player_uuid,
            scope,
            source,
        ))? + 1;
        store(lkjmc_store::player::insert_snapshot_with_metadata(
            client,
            lkjmc_store::player::NewSnapshot {
                id: Uuid::new_v4(),
                player_uuid,
                scope,
                revision,
                payload_format: &snapshot.payload_format,
                payload: &snapshot.payload,
                sha256: &snapshot.sha256,
                source_instance: source,
                metadata: json!({"restoredFrom": snapshot_id.to_string(), "restoredRevision": snapshot.revision}),
            },
        ))?;
        store(lkjmc_store::player::upsert_lease(
            client,
            player_uuid,
            scope,
            source,
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
