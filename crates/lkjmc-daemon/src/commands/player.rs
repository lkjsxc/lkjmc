use lkjmc_core::command::CommandEnvelope;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, store, with_connection};

type Response = lkjmc_core::command::CommandResponse;

pub fn handle(state: &AppState, request: CommandEnvelope) -> Response {
    match request.command.as_str() {
        "player.inspect" => inspect(state, request),
        "player.load" => load(state, request),
        "player.recovery.report" => recovery_report(state, request),
        "player.snapshot" => snapshot(state, request),
        "player.transfer.saved" => removed(request, "audit-only transfer command was removed"),
        _ => api::error(request, "command.unknown", "unknown player command", false),
    }
}

fn inspect(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request.body, "playerUuid")?;
        let scope = scope(&request.body);
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
                "latestRevision": latest.as_ref().map(|snapshot| snapshot.revision),
                "schema": latest.as_ref().map(|_| "lkjmc-profile-one")
            }),
        ))
    })
}

fn load(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request.body, "playerUuid")?;
        let Some(snapshot) = store(lkjmc_store::player::latest_snapshot(
            client,
            player_uuid,
            scope(&request.body),
        ))?
        else {
            return Ok(api::ok(request, json!({"found": false})));
        };
        Ok(api::ok(
            request,
            json!({
                "found": true,
                "playerUuid": snapshot.player_uuid.to_string(),
                "revision": snapshot.revision,
                "sessionRevision": snapshot.session_revision,
                "leaseFence": snapshot.lease_fence,
                "correlationId": snapshot.correlation_id.to_string(),
                "sha256": snapshot.sha256,
                "profile": snapshot.envelope
            }),
        ))
    })
}

fn snapshot(state: &AppState, request: CommandEnvelope) -> Response {
    let profile = canonical_request_profile(&request.body);
    let canonical = match profile {
        Ok(profile) => profile,
        Err(error) => return api::error(request, "command.invalid", &error, false),
    };
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request.body, "playerUuid")?;
        let result = store(lkjmc_store::player::write_snapshot(
            client,
            lkjmc_store::player::NewSnapshot {
                id: parse_uuid(&request.body, "snapshotId")?,
                player_uuid,
                scope: scope(&request.body),
                session_id: parse_uuid(&request.body, "sessionId")?,
                expected_session_revision: body_i64(&request.body, "expectedSessionRevision")?,
                expected_lease_fence: body_i64(&request.body, "expectedLeaseFence")?,
                expected_snapshot_revision: body_i64(&request.body, "expectedSnapshotRevision")?,
                correlation_id: parse_uuid(&request.body, "correlationId")?,
                source_instance: &body_string(&request.body, "sourceInstance")?,
                profile_json: &canonical.json,
            },
        ))?;
        Ok(api::ok(
            request,
            json!({
                "playerUuid": player_uuid.to_string(),
                "revision": result.revision,
                "sha256": result.sha256,
                "replay": result.replay
            }),
        ))
    })
}

fn removed(request: CommandEnvelope, message: &str) -> Response {
    api::error(request, "command.denied_unproved", message, false)
}

fn recovery_report(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request.body, "playerUuid")?;
        crate::support::audit_helpers::audit(
            client,
            &request,
            "player.recovery.report",
            "player",
            &player_uuid.to_string(),
            "recorded",
        )?;
        Ok(api::ok(
            request,
            json!({"playerUuid": player_uuid.to_string(), "recorded": true}),
        ))
    })
}

fn canonical_request_profile(
    body: &Value,
) -> Result<lkjmc_core::profile_validation::CanonicalProfile, String> {
    body_string(body, "profileJson").and_then(|profile| {
        lkjmc_core::profile_validation::canonical_profile(profile.as_bytes())
            .map_err(|error| format!("invalid profile: {error}"))
    })
}

fn scope(body: &Value) -> &str {
    body.get("scope")
        .and_then(Value::as_str)
        .unwrap_or("profile")
}

fn body_i64(body: &Value, field: &'static str) -> Result<i64, String> {
    body.get(field)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("missing or invalid {field}"))
}

fn parse_uuid(body: &Value, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(body, field)?).map_err(|error| error.to_string())
}

#[cfg(test)]
#[path = "player_tests.rs"]
mod tests;
