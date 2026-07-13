use std::time::{SystemTime, UNIX_EPOCH};

use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::{json, Value};

use crate::app::AppState;
use crate::dispatch as api;

pub fn status(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    status_response(request, status_body(state))
}

fn status_response(
    request: CommandEnvelope,
    result: Result<Value, lkjmc_store::error::StoreError>,
) -> CommandResponse {
    match result {
        Ok(body) => api::ok(request, body),
        Err(error) => api::database_error(request, error),
    }
}

fn status_body(state: &AppState) -> Result<Value, lkjmc_store::error::StoreError> {
    let (database, counts) = database_status(state)?;
    Ok(json!({
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
    }))
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

fn database_status(state: &AppState) -> Result<(Value, Value), lkjmc_store::error::StoreError> {
    let empty_counts = json!({"instances": null, "activeSessions": null, "jarAssets": null, "presenceRecords": null});
    if state.database_url().is_none() {
        return Ok((
            json!({"configured": false, "connected": null, "poolSize": null}),
            empty_counts,
        ));
    }
    let mut client = state.request_database_connection()?;
    let counts = lkjmc_store::status::counts(&mut client)?;
    Ok((
        json!({"configured": true, "connected": true, "poolSize": state.database_pool_size()}),
        json!({
            "instances": counts.instances,
            "activeSessions": counts.active_sessions,
            "jarAssets": counts.jar_assets,
            "presenceRecords": counts.presence_records
        }),
    ))
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
    fn status_timeout_outcome_pass_is_never_success() -> Result<(), String> {
        for code in [
            postgres::error::SqlState::QUERY_CANCELED,
            postgres::error::SqlState::LOCK_NOT_AVAILABLE,
        ] {
            let response = status_response(
                request("status").map_err(|error| error.to_string())?,
                Err(lkjmc_store::error::StoreError::Postgres {
                    message: "ignored".to_string(),
                    sql_state: Some(code),
                }),
            );
            assert!(!response.ok);
            assert_eq!(
                response.error.map(|error| error.code),
                Some("command.deadline_exceeded".into())
            );
        }
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
