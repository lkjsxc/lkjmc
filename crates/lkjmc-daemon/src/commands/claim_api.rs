use lkjmc_core::command::{CommandEnvelope, CommandResponse};

use crate::app::AppState;
use crate::dispatch as api;

pub fn handle(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    match request.command.as_str() {
        "claim.create" => crate::commands::claim_create::create(state, request),
        "claim.delete" => crate::commands::claim_create::delete(state, request),
        "claim.list" => crate::commands::claim_read::list(state, request),
        "claim.snapshot" => crate::commands::claim_read::snapshot(state, request),
        "claim.trust" => crate::commands::claim_trust::trust(state, request),
        "claim.untrust" => crate::commands::claim_trust::untrust(state, request),
        _ => api::error(request, "command.unknown", "unknown claim command", false),
    }
}

#[cfg(test)]
mod tests {
    use lkjmc_core::command::{Actor, ActorKind};
    use lkjmc_core::id::CommandId;
    use serde_json::{json, Value};
    use uuid::Uuid;

    use super::*;

    const OWNER: &str = "00000000-0000-0000-0000-000000000301";
    const TRUSTED: &str = "00000000-0000-0000-0000-000000000302";

    #[test]
    fn claim_dispatch_round_trips_when_database_configured() -> Result<(), String> {
        let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
            return Ok(());
        };
        let mut guard = reset_and_migrate(&database_url)?;
        let state = state(database_url);
        let created = call(&state, "claim.create", create_body())?;
        let claim_id = text(&created, "claimId")?;
        assert_eq!(count(&mut guard, "player_claims")?, 1);
        assert_eq!(count(&mut guard, "player_identities")?, 1);
        assert_eq!(count(&mut guard, "audit_events")?, 1);
        call(&state, "claim.trust", trust_body())?;
        let listed = call(&state, "claim.list", json!({"ownerUuid": OWNER}))?;
        assert_eq!(listed["claims"].as_array().map(Vec::len), Some(1));
        let snapshot = call(&state, "claim.snapshot", json!({"instanceId": "survival"}))?;
        assert_eq!(snapshot["chunks"].as_array().map(Vec::len), Some(1));
        assert_eq!(snapshot["chunks"][0]["claimId"], json!(claim_id));
        assert_eq!(snapshot["chunks"][0]["trusts"][0]["uuid"], json!(TRUSTED));
        call(
            &state,
            "claim.delete",
            json!({"claimId": claim_id, "operator": true}),
        )?;
        let snapshot = call(&state, "claim.snapshot", json!({"instanceId": "survival"}))?;
        assert_eq!(snapshot["chunks"].as_array().map(Vec::len), Some(0));
        guard
            .batch_execute("select pg_advisory_unlock(752647)")
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn claim_create_rolls_back_when_chunk_insert_fails() -> Result<(), String> {
        let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
            return Ok(());
        };
        let mut guard = reset_and_migrate(&database_url)?;
        let owner_uuid = Uuid::parse_str(OWNER).map_err(|error| error.to_string())?;
        let existing = lkjmc_store::claims::NewClaim {
            id: Uuid::new_v4(),
            owner_uuid,
            owner_name: "Owner",
            name: "Existing",
            instance_id: "survival",
            world_name: "world",
            chunk_x: 1,
            chunk_z: 2,
        };
        lkjmc_store::claims::create_claim(&mut guard, existing)
            .map_err(|error| error.to_string())?;
        let state = state(database_url);
        assert!(call(&state, "claim.create", create_body()).is_err());
        assert_eq!(count(&mut guard, "player_claims")?, 1);
        assert_eq!(count(&mut guard, "claim_chunks")?, 1);
        assert_eq!(count(&mut guard, "player_achievements")?, 0);
        assert_eq!(count(&mut guard, "audit_events")?, 0);
        Ok(())
    }

    fn reset_and_migrate(database_url: &str) -> Result<postgres::Client, String> {
        let mut client =
            lkjmc_store::pool::connect_single(database_url).map_err(|error| error.to_string())?;
        client
            .batch_execute(
                "select pg_advisory_lock(752647); drop schema public cascade; create schema public",
            )
            .map_err(|error| error.to_string())?;
        lkjmc_store::migrate::apply(&mut client).map_err(|error| error.to_string())?;
        Ok(client)
    }

    fn count(client: &mut postgres::Client, table: &str) -> Result<i64, String> {
        let row = client
            .query_one(&format!("select count(*)::bigint from {table}"), &[])
            .map_err(|error| error.to_string())?;
        Ok(row.get(0))
    }

    fn state(database_url: String) -> AppState {
        AppState::with_config_path(
            Some(database_url),
            8,
            "/tmp/lkjmc-config".to_string(),
            "/tmp/lkjmc-logs".to_string(),
            "/tmp/lkjmc-jars".to_string(),
            "/tmp/lkjmc-data".to_string(),
            None,
            None,
            None,
        )
    }

    fn call(state: &AppState, command: &str, body: Value) -> Result<Value, String> {
        let response = crate::dispatch::dispatch(state, request(command, body)?);
        if response.ok {
            return response
                .body
                .ok_or_else(|| "missing response body".to_string());
        }
        Err(response
            .error
            .map(|error| format!("{}: {}", error.code, error.message))
            .unwrap_or_else(|| "unknown error".to_string()))
    }

    fn request(command: &str, body: Value) -> Result<CommandEnvelope, String> {
        Ok(CommandEnvelope {
            request_id: CommandId::parse("request id", command)
                .map_err(|error| error.to_string())?,
            actor: Actor {
                kind: ActorKind::Cli,
                name: "claim-test".to_string(),
            },
            command: command.to_string(),
            body,
        })
    }

    fn create_body() -> Value {
        json!({
            "ownerUuid": OWNER,
            "ownerName": "Owner",
            "name": "Base",
            "instanceId": "survival",
            "worldName": "world",
            "chunkX": 1,
            "chunkZ": 2
        })
    }

    fn trust_body() -> Value {
        json!({
            "ownerUuid": OWNER,
            "trustedUuid": TRUSTED,
            "trustedName": "Friend",
            "instanceId": "survival",
            "worldName": "world",
            "chunkX": 1,
            "chunkZ": 2
        })
    }

    fn text<'a>(body: &'a Value, field: &str) -> Result<&'a str, String> {
        body.get(field)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("missing field: {field}"))
    }
}
