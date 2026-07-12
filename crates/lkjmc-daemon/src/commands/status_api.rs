use std::time::{SystemTime, UNIX_EPOCH};

use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::{json, Value};

use crate::app::AppState;
use crate::dispatch as api;

pub fn status(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    api::ok(request, status_body(state))
}

fn status_body(state: &AppState) -> Value {
    let (database, counts) = database_status(state);
    json!({
        "daemon": "running",
        "startedAtUnixSeconds": unix_seconds(state.started_at()),
        "uptimeSeconds": uptime_seconds(state.started_at()),
        "database": database,
        "counts": counts,
        "roots": {
            "config": state.config_root(),
            "data": state.data_root(),
            "log": state.log_root(),
            "jar": state.jar_root()
        },
        "socket": {"path": state.socket_path()},
        "http": match state.http_listener() {
            Some(address) => json!({"enabled": true, "address": address}),
            None => json!({"enabled": false})
        },
        "runtime": runtime_status(state),
        "commandLifecycle": {
            "admissionLimit": crate::command_lifecycle::ADMISSION_LIMIT,
            "deadlineSeconds": crate::command_lifecycle::DEADLINE.as_secs(),
            "queue": "none",
            "externalEffects": "denied-unproved"
        },
        "reconciler": {"enabled": state.reconciler_enabled()}
    })
}

fn runtime_status(state: &AppState) -> Value {
    match state.runtime_adapter_name() {
        Ok(adapter) => json!({
            "adapter": adapter,
            "externalEffects": "denied-unproved"
        }),
        Err(_) => json!({"adapter": "unknown", "error": "runtime lock poisoned"}),
    }
}

fn database_status(state: &AppState) -> (Value, Value) {
    let empty_counts = json!({"instances": null, "activeSessions": null, "jarAssets": null, "presenceRecords": null});
    let Some(database_url) = state.database_url() else {
        return (
            json!({"configured": false, "connected": null, "poolSize": null}),
            empty_counts,
        );
    };
    let Some(pool) = state.database_pool() else {
        return (
            json!({"configured": true, "connected": false}),
            empty_counts,
        );
    };
    let mut client = match pool.get() {
        Ok(client) => client,
        Err(error) => {
            return (
                json!({
                    "configured": true,
                    "connected": false,
                    "error": sanitize(&error.to_string(), &database_url)
                }),
                empty_counts,
            )
        }
    };
    match lkjmc_store::status::counts(&mut client) {
        Ok(counts) => (
            json!({"configured": true, "connected": true, "poolSize": state.database_pool_size()}),
            json!({
                "instances": counts.instances,
                "activeSessions": counts.active_sessions,
                "jarAssets": counts.jar_assets,
                "presenceRecords": counts.presence_records
            }),
        ),
        Err(error) => (
            json!({
                "configured": true,
                "connected": true,
                "error": sanitize(&error.to_string(), &database_url)
            }),
            empty_counts,
        ),
    }
}
fn sanitize(message: &str, secret: &str) -> String {
    message.replace(secret, "[redacted-database-url]")
}

fn unix_seconds(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

fn uptime_seconds(started_at: SystemTime) -> u64 {
    SystemTime::now()
        .duration_since(started_at)
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use lkjmc_core::command::{Actor, ActorKind};
    use lkjmc_core::id::CommandId;

    use super::*;

    #[test]
    fn status_reports_no_database_configuration() -> Result<(), String> {
        let response = status(
            &state(None),
            request("status").map_err(|error| error.to_string())?,
        );
        let body = response
            .body
            .ok_or_else(|| "status body missing".to_string())?;
        assert!(response.ok);
        assert_eq!(body["daemon"], json!("running"));
        assert_eq!(body["database"]["configured"], json!(false));
        assert_eq!(body["counts"]["instances"], Value::Null);
        assert_eq!(body["runtime"]["adapter"], json!("local-process"));
        assert_eq!(body["runtime"]["externalEffects"], json!("denied-unproved"));
        assert_eq!(body["commandLifecycle"]["admissionLimit"], json!(8));
        Ok(())
    }

    #[test]
    fn status_reports_test_database_when_configured() -> Result<(), String> {
        let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
            return Ok(());
        };
        let _lock = crate::test_database::migrate(&database_url)?;
        let response = status(
            &state(Some(database_url)),
            request("status").map_err(|error| error.to_string())?,
        );
        let body = response
            .body
            .ok_or_else(|| "status body missing".to_string())?;
        assert_eq!(body["database"]["configured"], json!(true));
        assert_eq!(body["database"]["connected"], json!(true));
        assert!(body["counts"]["instances"].as_i64().is_some());
        assert!(body["counts"]["activeSessions"].as_i64().is_some());
        assert!(body["counts"]["jarAssets"].as_i64().is_some());
        Ok(())
    }

    fn state(database_url: Option<String>) -> AppState {
        AppState::with_config_path(
            database_url,
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

    fn request(command: &str) -> Result<CommandEnvelope, lkjmc_core::error::IdError> {
        Ok(CommandEnvelope {
            request_id: CommandId::parse("request id", "test")?,
            actor: Actor {
                kind: ActorKind::Cli,
                name: "test".to_string(),
            },
            command: command.to_string(),
            body: json!({}),
        })
    }
}
