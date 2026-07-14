use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use serde_json::{json, Value};

use super::{status, status_response};
use crate::app::AppState;

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
    assert_eq!(
        body["runtime"]["coordination"],
        json!("per-instance-fenced")
    );
    assert_eq!(body["commandLifecycle"]["admissionLimit"], json!(8));
    assert_eq!(body["syncMaintenance"]["running"], json!(false));
    assert_eq!(body["syncMaintenance"]["singletonCount"], json!(0));
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
