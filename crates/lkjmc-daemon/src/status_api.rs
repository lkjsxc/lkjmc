use std::time::{SystemTime, UNIX_EPOCH};

use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::{json, Value};

use crate::api;
use crate::app::AppState;

pub fn status(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    api::ok(request, status_body(state))
}

fn status_body(state: &AppState) -> Value {
    let (database, counts) = database_status(state.database_url());
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
        "reconciler": {"enabled": state.reconciler_enabled()}
    })
}

fn runtime_status(state: &AppState) -> Value {
    match (state.runtime_adapter_name(), state.runtime_capabilities()) {
        (Ok(adapter), Ok(capabilities)) => json!({
            "adapter": adapter,
            "capabilities": {
                "start": capabilities.start,
                "stop": capabilities.stop,
                "restart": capabilities.restart,
                "delete": capabilities.delete,
                "logs": capabilities.logs,
                "recover": capabilities.recover,
                "readiness": capabilities.readiness
            }
        }),
        _ => json!({"adapter": "unknown", "error": "runtime lock poisoned"}),
    }
}

fn database_status(database_url: Option<String>) -> (Value, Value) {
    let empty_counts = json!({"instances": null, "activeSessions": null, "jarAssets": null, "presenceRecords": null});
    let Some(database_url) = database_url else {
        return (
            json!({"configured": false, "connected": null}),
            empty_counts,
        );
    };
    let mut client = match lkjmc_store::pool::connect(&database_url) {
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
            json!({"configured": true, "connected": true}),
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
        assert_eq!(body["runtime"]["capabilities"]["start"], json!(true));
        Ok(())
    }

    #[test]
    fn status_reports_test_database_when_configured() -> Result<(), String> {
        let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
            return Ok(());
        };
        let _lock = apply_migrations(&database_url)?;
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

    fn apply_migrations(database_url: &str) -> Result<postgres::Client, String> {
        let mut client =
            lkjmc_store::pool::connect(database_url).map_err(|error| error.to_string())?;
        client
            .batch_execute("select pg_advisory_lock(752647)")
            .map_err(|error| error.to_string())?;
        lkjmc_store::migrate::apply(&mut client).map_err(|error| error.to_string())?;
        Ok(client)
    }
}
