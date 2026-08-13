use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use serde_json::{json, Value};

use super::{instance_status, status, status_response};
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
    assert_eq!(body["build"]["version"], json!("0.1.0-alpha.1"));
    assert!(body["build"]["commit"].is_string());
    assert!(body["build"]["dirty"].is_boolean() || body["build"]["dirty"].is_null());
    assert_eq!(body["database"]["configured"], json!(false));
    assert_eq!(body["counts"]["instances"], Value::Null);
    assert_eq!(body["instances"], Value::Null);
    assert_eq!(
        body["instanceSnapshot"]["source"],
        json!("postgresql-latest-observation")
    );
    assert_eq!(body["instanceSnapshot"]["runtimeRefresh"], json!(false));
    assert_eq!(body["instanceSnapshot"]["limit"], json!(32));
    assert_eq!(body["instanceSnapshot"]["truncated"], json!(false));
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
fn stale_missing_stopped_and_proxy_rows_do_not_claim_joinability() {
    let mut missing = instance_record("folia", "running");
    missing.heartbeat_ready = None;
    missing.heartbeat_age_seconds = None;
    missing.proxy_registered = None;
    missing.proxy_registration_age_seconds = None;
    let missing = instance_status(missing);
    assert_eq!(missing["ready"], Value::Null);
    assert_eq!(missing["readinessSource"], json!("unavailable"));
    assert_eq!(missing["proxyRegistered"], Value::Null);
    assert_eq!(missing["joinable"], json!(false));
    assert_eq!(missing["joinDisabledReason"], json!("heartbeat-missing"));

    let mut stale = instance_record("folia", "running");
    stale.heartbeat_age_seconds = Some(31);
    let stale = instance_status(stale);
    assert_eq!(stale["ready"], json!(false));
    assert_eq!(stale["joinable"], json!(false));
    assert_eq!(stale["joinDisabledReason"], json!("heartbeat-stale"));

    let mut stale_registration = instance_record("folia", "running");
    stale_registration.proxy_registration_age_seconds = Some(31);
    let stale_registration = instance_status(stale_registration);
    assert_eq!(stale_registration["proxyRegistered"], json!(false));
    assert_eq!(
        stale_registration["joinDisabledReason"],
        json!("proxy-registration-stale")
    );

    let stopped = instance_status(instance_record("folia", "stopped"));
    assert_eq!(stopped["joinable"], json!(false));
    assert_eq!(
        stopped["joinDisabledReason"],
        json!("desired-state-not-running")
    );

    let proxy = instance_status(instance_record("velocity", "running"));
    assert_eq!(proxy["ready"], Value::Null);
    assert_eq!(proxy["joinable"], json!(false));
    assert_eq!(proxy["joinDisabledReason"], json!("not-a-backend"));
}

#[test]
#[ignore = "requires LKJMC_STORE_TEST_DATABASE_URL"]
fn status_includes_truthful_instance_snapshot() -> Result<(), String> {
    let database_url = std::env::var("LKJMC_STORE_TEST_DATABASE_URL")
        .map_err(|_| "LKJMC_STORE_TEST_DATABASE_URL is required".to_string())?;
    let mut database = crate::test_database::migrate(&database_url)?;
    let empty =
        lkjmc_store::status::snapshot(database.client_mut()).map_err(|error| error.to_string())?;
    assert_eq!(empty.counts.instances, 0);
    assert!(empty.instances.is_empty());
    assert!(!empty.instances_truncated);
    lkjmc_store::instance::insert(
        database.client_mut(),
        "survival",
        None,
        "folia",
        "running",
        &json!({"serverPort": 25567, "connectHost": "127.0.0.1"}),
    )
    .map_err(|error| error.to_string())?;
    let long_message = "x".repeat(300);
    lkjmc_store::instance::upsert_observation(
        database.client_mut(),
        "survival",
        "process-healthy",
        Some(123),
        true,
        Some(&long_message),
    )
    .map_err(|error| error.to_string())?;
    lkjmc_store::instance_presence::upsert_heartbeat(
        database.client_mut(),
        lkjmc_store::instance_presence::PresenceHeartbeat {
            instance_id: "survival",
            player_count: Some(0),
            max_players: Some(20),
            ready: true,
            implementation: Some("test"),
        },
    )
    .map_err(|error| error.to_string())?;
    lkjmc_store::proxy_registration::report(
        database.client_mut(),
        &[lkjmc_store::proxy_registration::RegistrationReport {
            instance_id: "survival",
            connect_host: "127.0.0.1",
            connect_port: 25567,
            registered: true,
            failure_reason: None,
        }],
    )
    .map_err(|error| error.to_string())?;
    for index in 0..32 {
        lkjmc_store::instance::insert(
            database.client_mut(),
            &format!("z-extra-{index:02}"),
            None,
            "folia",
            "stopped",
            &json!({"serverPort": 26000 + index}),
        )
        .map_err(|error| error.to_string())?;
    }

    let state = state(Some(database.url().to_string()));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    let response = runtime.block_on(async {
        let admission = state
            .admit_request()
            .ok_or_else(|| "status admission unavailable".to_string())?;
        let request = request("status").map_err(|error| error.to_string())?;
        let state = state.clone();
        admission
            .run_blocking(move || crate::dispatch::dispatch(&state, request))
            .await
            .map_err(|_| "status worker did not complete".to_string())
    })?;
    if !response.ok {
        return Err(format!("status failed: {:?}", response.error));
    }
    let body = response.body.ok_or("status body missing")?;
    let instances = body["instances"]
        .as_array()
        .ok_or("status instances missing")?;
    assert!(response.ok);
    assert_eq!(body["counts"]["instances"], json!(33));
    assert_eq!(instances.len(), 32);
    let ids = instances
        .iter()
        .map(|row| row["id"].as_str().unwrap_or_default())
        .collect::<Vec<_>>();
    assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(ids.first(), Some(&"survival"));
    assert_eq!(ids.last(), Some(&"z-extra-30"));
    assert!(!ids.contains(&"z-extra-31"));
    let survival = instances
        .iter()
        .find(|row| row["id"] == json!("survival"))
        .ok_or("survival status missing")?;
    assert_eq!(survival["desiredState"], json!("running"));
    assert_eq!(survival["observedState"], json!("process-healthy"));
    assert_eq!(survival["processHealthy"], json!(true));
    assert_eq!(survival["ready"], json!(true));
    assert_eq!(survival["joinable"], json!(true));
    assert!(survival["observationAgeSeconds"].is_number());
    assert_eq!(survival["diagnosticsTruncated"], json!(true));
    assert_eq!(
        survival["observationMessage"]
            .as_str()
            .unwrap_or_default()
            .chars()
            .count(),
        256
    );
    assert_eq!(body["instanceSnapshot"]["truncated"], json!(true));
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

fn instance_record(kind: &str, desired_state: &str) -> lkjmc_store::status::InstanceStatus {
    lkjmc_store::status::InstanceStatus {
        id: "instance".to_string(),
        id_truncated: false,
        kind: kind.to_string(),
        desired_state: desired_state.to_string(),
        observed_state: Some("process-healthy".to_string()),
        process_healthy: Some(true),
        pid: Some(123),
        observation_message: None,
        observation_age_seconds: Some(2),
        configured_host: "127.0.0.1".to_string(),
        configured_port: Some(25567),
        heartbeat_ready: Some(true),
        heartbeat_age_seconds: Some(2),
        registered_host: Some("127.0.0.1".to_string()),
        registered_port: Some(25567),
        proxy_registered: Some(true),
        proxy_failure_reason: None,
        proxy_registration_age_seconds: Some(2),
        diagnostics_truncated: false,
    }
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
