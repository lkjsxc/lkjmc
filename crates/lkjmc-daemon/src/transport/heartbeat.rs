use std::collections::BTreeMap;
use std::net::IpAddr;

use axum::body::{to_bytes, Body};
use axum::extract::{Extension, State};
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use tokio::time::timeout_at;

use crate::app::{AppState, BlockingError, RequestAdmission};
use crate::authz::AuthenticatedSubject;

const BODY_LIMIT: usize = 32 * 1024;
const MAX_BACKENDS: usize = 64;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct VelocityHeartbeat {
    registrations: Vec<RegistrationObservation>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RegistrationObservation {
    instance_id: String,
    connect_host: String,
    connect_port: i32,
    registered: bool,
    #[serde(default)]
    failure_reason: Option<String>,
}

pub async fn heartbeat(
    State(state): State<AppState>,
    Extension(subject): Extension<AuthenticatedSubject>,
    Extension(admission): Extension<RequestAdmission>,
    request: Request<Body>,
) -> Response {
    let Some((surface, instance_id)) = subject.heartbeat_identity() else {
        return error(StatusCode::FORBIDDEN, "auth.policy_denied");
    };
    let surface = surface.to_string();
    let instance_id = instance_id.to_string();
    let body = match timeout_at(
        admission.deadline(),
        to_bytes(request.into_body(), BODY_LIMIT),
    )
    .await
    {
        Ok(Ok(body)) => body,
        Ok(Err(_)) => return error(StatusCode::BAD_REQUEST, "heartbeat.body_invalid"),
        Err(_) => return error(StatusCode::REQUEST_TIMEOUT, "heartbeat.deadline_exceeded"),
    };
    let velocity = match parse_body(&surface, &body) {
        Ok(payload) => payload,
        Err(()) => return error(StatusCode::BAD_REQUEST, "heartbeat.body_invalid"),
    };

    let work = move || record(&state, &surface, &instance_id, velocity.as_ref());
    match admission.run_blocking(work).await {
        Ok(Ok(())) => StatusCode::NO_CONTENT.into_response(),
        Ok(Err(HeartbeatError::Invalid)) => {
            error(StatusCode::BAD_REQUEST, "heartbeat.body_invalid")
        }
        Ok(Err(HeartbeatError::Denied)) => {
            error(StatusCode::FORBIDDEN, "heartbeat.identity_denied")
        }
        Ok(Err(HeartbeatError::Store(store_error))) if store_error.is_deadline() => {
            error(StatusCode::REQUEST_TIMEOUT, "heartbeat.deadline_exceeded")
        }
        Ok(Err(HeartbeatError::Store(_))) | Err(BlockingError::Join) => {
            error(StatusCode::SERVICE_UNAVAILABLE, "heartbeat.unavailable")
        }
        Err(BlockingError::Deadline) => {
            error(StatusCode::REQUEST_TIMEOUT, "heartbeat.deadline_exceeded")
        }
    }
}

fn parse_body(surface: &str, body: &[u8]) -> Result<Option<VelocityHeartbeat>, ()> {
    match surface {
        "paper" if body.is_empty() => Ok(None),
        "velocity" if !body.is_empty() => {
            let payload: VelocityHeartbeat = serde_json::from_slice(body).map_err(|_| ())?;
            if payload.registrations.is_empty() || payload.registrations.len() > MAX_BACKENDS {
                return Err(());
            }
            Ok(Some(payload))
        }
        _ => Err(()),
    }
}

fn record(
    state: &AppState,
    surface: &str,
    instance_id: &str,
    velocity: Option<&VelocityHeartbeat>,
) -> Result<(), HeartbeatError> {
    let mut client = state
        .request_database_connection()
        .map_err(HeartbeatError::Store)?;
    let mut transaction = client.transaction().map_err(store_error)?;
    let kind = transaction
        .query_opt("select kind from instances where id = $1", &[&instance_id])
        .map_err(store_error)?
        .map(|row| row.get::<_, String>(0))
        .ok_or(HeartbeatError::Denied)?;
    if !surface_matches_kind(surface, &kind) {
        return Err(HeartbeatError::Denied);
    }
    lkjmc_store::instance_presence::upsert_heartbeat_in(
        &mut transaction,
        lkjmc_store::instance_presence::PresenceHeartbeat {
            instance_id,
            player_count: None,
            max_players: None,
            ready: true,
            implementation: Some(&kind),
        },
    )
    .map_err(HeartbeatError::Store)?;
    if surface == "velocity" {
        record_observed_proxy_registrations(
            &mut transaction,
            velocity.ok_or(HeartbeatError::Invalid)?,
        )?;
    }
    transaction.commit().map_err(store_error)?;
    Ok(())
}

fn record_observed_proxy_registrations(
    transaction: &mut postgres::Transaction<'_>,
    payload: &VelocityHeartbeat,
) -> Result<(), HeartbeatError> {
    let rows = transaction
        .query(
            "select id, kind, config from instances
             where kind <> 'velocity' order by id",
            &[],
        )
        .map_err(store_error)?;
    if rows.is_empty() {
        return Err(HeartbeatError::Invalid);
    }
    let mut observed = BTreeMap::new();
    for entry in &payload.registrations {
        if !lkjmc_core::validation::is_kebab_id(&entry.instance_id)
            || entry.connect_port <= 0
            || entry.connect_port > 65_535
            || entry
                .connect_host
                .parse::<IpAddr>()
                .map_or(true, |address| !address.is_loopback())
            || !valid_registration_result(entry)
            || observed.insert(entry.instance_id.as_str(), entry).is_some()
        {
            return Err(HeartbeatError::Invalid);
        }
    }
    if observed.len() != rows.len() {
        return Err(HeartbeatError::Invalid);
    }
    let mut registrations = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.get(0);
        let kind: String = row.get(1);
        let config: serde_json::Value = row.get(2);
        let port = config
            .get("serverPort")
            .and_then(serde_json::Value::as_i64)
            .and_then(|value| i32::try_from(value).ok())
            .filter(|value| (1..=65_535).contains(value))
            .ok_or(HeartbeatError::Invalid)?;
        let host = config
            .pointer("/properties/server-ip")
            .and_then(serde_json::Value::as_str)
            .ok_or(HeartbeatError::Invalid)?;
        let expected_host = host
            .parse::<IpAddr>()
            .ok()
            .filter(IpAddr::is_loopback)
            .ok_or(HeartbeatError::Invalid)?;
        if !matches!(kind.as_str(), "paper" | "folia" | "purpur") {
            return Err(HeartbeatError::Invalid);
        }
        let entry = observed.get(id.as_str()).ok_or(HeartbeatError::Invalid)?;
        let observed_host = entry
            .connect_host
            .parse::<IpAddr>()
            .map_err(|_| HeartbeatError::Invalid)?;
        if entry.connect_port != port || observed_host != expected_host {
            return Err(HeartbeatError::Invalid);
        }
        registrations.push((id, host.to_string(), port, entry));
    }
    let reports = registrations
        .iter()
        .map(
            |(id, host, port, entry)| lkjmc_store::proxy_registration::RegistrationReport {
                instance_id: id,
                connect_host: host,
                connect_port: *port,
                registered: entry.registered,
                failure_reason: entry.failure_reason.as_deref(),
            },
        )
        .collect::<Vec<_>>();
    lkjmc_store::proxy_registration::report_in(transaction, &reports).map_err(HeartbeatError::Store)
}

fn valid_registration_result(entry: &RegistrationObservation) -> bool {
    matches!(
        (entry.registered, entry.failure_reason.as_deref()),
        (true, None) | (false, Some("missing-registration" | "route-mismatch"))
    )
}

fn surface_matches_kind(surface: &str, kind: &str) -> bool {
    match surface {
        "velocity" => kind == "velocity",
        "paper" => matches!(kind, "paper" | "folia" | "purpur"),
        _ => false,
    }
}

fn store_error(error: postgres::Error) -> HeartbeatError {
    HeartbeatError::Store(lkjmc_store::error::StoreError::from(error))
}

enum HeartbeatError {
    Denied,
    Invalid,
    Store(lkjmc_store::error::StoreError),
}

fn error(status: StatusCode, code: &str) -> Response {
    (status, Json(json!({"ok": false, "error": {"code": code}}))).into_response()
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::Request;
    use tower::Service;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn only_matching_platform_kinds_are_accepted() {
        assert!(surface_matches_kind("velocity", "velocity"));
        assert!(surface_matches_kind("paper", "paper"));
        assert!(surface_matches_kind("paper", "folia"));
        assert!(surface_matches_kind("paper", "purpur"));
        assert!(!surface_matches_kind("velocity", "folia"));
        assert!(!surface_matches_kind("paper", "velocity"));
    }

    #[tokio::test]
    async fn body_collection_uses_the_shared_absolute_deadline() -> Result<(), String> {
        let state = crate::app::Admission::with_test_deadline(
            std::time::Duration::from_millis(1),
            empty_state,
        );
        let admission = state.admit_request().ok_or("admission unavailable")?;
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        let response = super::heartbeat(
            State(state),
            Extension(subject("paper", "quartz-world")),
            Extension(admission),
            request("unused", Body::empty())?,
        )
        .await;
        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
        Ok(())
    }

    #[test]
    fn heartbeat_database_lock_obeys_request_deadline() -> Result<(), String> {
        let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
            eprintln!("SKIP heartbeat deadline: LKJMC_STORE_TEST_DATABASE_URL is unset");
            return Ok(());
        };
        let mut database = crate::test_database::migrate(&database_url)?;
        lkjmc_store::instance::insert(
            database.client_mut(),
            "quartz-world",
            None,
            "folia",
            "running",
            &json!({"serverPort": 25566}),
        )
        .map_err(|error| error.to_string())?;
        let token = "heartbeat-lock-deadline-token";
        lkjmc_store::daemon_token::insert(
            database.client_mut(),
            Uuid::new_v4(),
            &lkjmc_core::security::token_hash(token),
            "paper",
            "instance",
            "quartz-world",
            &["lkjmc.instance.heartbeat".to_string()],
            3600,
        )
        .map_err(|error| error.to_string())?;
        let worker_url = database.url().to_string();
        let state = crate::app::Admission::with_test_deadline(
            std::time::Duration::from_millis(100),
            || database_state(worker_url),
        );
        let mut lock = database
            .client_mut()
            .transaction()
            .map_err(|error| error.to_string())?;
        lock.batch_execute("lock table instance_presence in access exclusive mode")
            .map_err(|error| error.to_string())?;
        let response = call(state, request(token, Body::empty())?)?;
        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
        drop(lock);
        assert!(
            lkjmc_store::instance_presence::get(database.client_mut(), "quartz-world")
                .map_err(|error| error.to_string())?
                .is_none()
        );
        Ok(())
    }

    #[test]
    fn scoped_heartbeats_preserve_instance_identity_and_observed_proxy_state() -> Result<(), String>
    {
        let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
            eprintln!("SKIP heartbeat integration: LKJMC_STORE_TEST_DATABASE_URL is unset");
            return Ok(());
        };
        let mut database = crate::test_database::migrate(&database_url)?;
        lkjmc_store::instance::insert(
            database.client_mut(),
            "quartz-world",
            None,
            "folia",
            "running",
            &json!({"serverPort": 25566, "properties": {"server-ip": "127.0.0.1"}}),
        )
        .map_err(|error| error.to_string())?;
        lkjmc_store::instance::insert(
            database.client_mut(),
            "ember-realm",
            None,
            "folia",
            "running",
            &json!({"serverPort": 25567, "properties": {"server-ip": "127.0.0.1"}}),
        )
        .map_err(|error| error.to_string())?;
        lkjmc_store::instance::insert(
            database.client_mut(),
            "edge-gateway",
            None,
            "velocity",
            "running",
            &json!({"serverPort": 25591}),
        )
        .map_err(|error| error.to_string())?;
        let token = "MiXeD-Case_heartbeat-token";
        let ember_token = "distinct-ember-heartbeat-token";
        let velocity_token = "distinct-velocity-heartbeat-token";
        let wrong_scope = "heartbeat-token-with-sync-only";
        lkjmc_store::daemon_token::insert(
            database.client_mut(),
            Uuid::new_v4(),
            &lkjmc_core::security::token_hash(token),
            "paper",
            "instance",
            "quartz-world",
            &["lkjmc.instance.heartbeat".to_string()],
            3600,
        )
        .map_err(|error| error.to_string())?;
        lkjmc_store::daemon_token::insert(
            database.client_mut(),
            Uuid::new_v4(),
            &lkjmc_core::security::token_hash(ember_token),
            "paper",
            "instance",
            "ember-realm",
            &["lkjmc.instance.heartbeat".to_string()],
            3600,
        )
        .map_err(|error| error.to_string())?;
        lkjmc_store::daemon_token::insert(
            database.client_mut(),
            Uuid::new_v4(),
            &lkjmc_core::security::token_hash(velocity_token),
            "velocity",
            "instance",
            "edge-gateway",
            &["lkjmc.instance.heartbeat".to_string()],
            3600,
        )
        .map_err(|error| error.to_string())?;
        lkjmc_store::daemon_token::insert(
            database.client_mut(),
            Uuid::new_v4(),
            &lkjmc_core::security::token_hash(wrong_scope),
            "paper",
            "instance",
            "quartz-world",
            &["lkjmc.sync.read".to_string()],
            3600,
        )
        .map_err(|error| error.to_string())?;
        for (id, pid) in [("quartz-world", 1001), ("ember-realm", 1002)] {
            lkjmc_store::instance::upsert_observation(
                database.client_mut(),
                id,
                "process-healthy",
                Some(pid),
                true,
                Some("test process"),
            )
            .map_err(|error| error.to_string())?;
        }
        let state = database_state(database.url().to_string());

        let denied = call(state.clone(), request(wrong_scope, Body::empty())?)?;
        assert_eq!(denied.status(), StatusCode::FORBIDDEN);
        let nonempty = call(state.clone(), request(token, Body::from("{}"))?)?;
        assert_eq!(nonempty.status(), StatusCode::BAD_REQUEST);
        assert!(
            lkjmc_store::instance_presence::get(database.client_mut(), "quartz-world")
                .map_err(|error| error.to_string())?
                .is_none()
        );

        let response = call(state, request(token, Body::empty())?)?;
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let presence = lkjmc_store::instance_presence::get(database.client_mut(), "quartz-world")
            .map_err(|error| error.to_string())?
            .ok_or("heartbeat presence missing")?;
        assert!(presence.ready);
        assert_eq!(presence.player_count, None);
        assert_eq!(presence.max_players, None);
        assert!(presence.heartbeat_age_seconds.is_some_and(|age| age <= 1));
        let ember_response = call(
            state_for(database.url()),
            request(ember_token, Body::empty())?,
        )?;
        assert_eq!(ember_response.status(), StatusCode::NO_CONTENT);

        let empty_velocity = call(
            state_for(database.url()),
            request(velocity_token, Body::empty())?,
        )?;
        assert_eq!(empty_velocity.status(), StatusCode::BAD_REQUEST);

        lkjmc_store::instance::update_config(
            database.client_mut(),
            "ember-realm",
            &json!({"serverPort": 0}),
        )
        .map_err(|error| error.to_string())?;
        let rolled_back = call(
            state_for(database.url()),
            request(velocity_token, velocity_body())?,
        )?;
        assert_eq!(rolled_back.status(), StatusCode::BAD_REQUEST);
        assert!(
            lkjmc_store::instance_presence::get(database.client_mut(), "edge-gateway")
                .map_err(|error| error.to_string())?
                .is_none()
        );
        assert!(
            lkjmc_store::proxy_registration::get(database.client_mut(), "quartz-world")
                .map_err(|error| error.to_string())?
                .is_none()
        );
        lkjmc_store::instance::update_config(
            database.client_mut(),
            "ember-realm",
            &json!({"serverPort": 25567, "properties": {"server-ip": "127.0.0.1"}}),
        )
        .map_err(|error| error.to_string())?;

        let proxy_response = call(
            state_for(database.url()),
            request(velocity_token, velocity_body())?,
        )?;
        assert_eq!(proxy_response.status(), StatusCode::NO_CONTENT);
        for (id, port) in [("quartz-world", 25566), ("ember-realm", 25567)] {
            let registration = lkjmc_store::proxy_registration::get(database.client_mut(), id)
                .map_err(|error| error.to_string())?
                .ok_or("proxy registration missing")?;
            assert!(registration.registered);
            assert_eq!(registration.connect_host, "127.0.0.1");
            assert_eq!(registration.connect_port, port);
            assert!(registration.age_seconds <= 1);
        }
        let snapshot = lkjmc_store::status::snapshot(database.client_mut())
            .map_err(|error| error.to_string())?;
        for row in snapshot
            .instances
            .into_iter()
            .filter(|row| row.kind == "folia")
        {
            let availability = crate::commands::instance_availability::evaluate(
                crate::commands::instance_availability::Input {
                    kind: &row.kind,
                    desired_state: &row.desired_state,
                    process_healthy: row.process_healthy,
                    connect_port: row.registered_port.map(i64::from).or(row.configured_port),
                    heartbeat_ready: row.heartbeat_ready,
                    heartbeat_age_seconds: row.heartbeat_age_seconds,
                    proxy_registration_desired: true,
                    proxy_registered: row.proxy_registered,
                    proxy_failure_reason: row.proxy_failure_reason.as_deref(),
                    proxy_registration_age_seconds: row.proxy_registration_age_seconds,
                },
            );
            assert!(availability.joinable, "{} was not joinable", row.id);
            assert_eq!(availability.ready, Some(true));
        }
        database
            .client_mut()
            .batch_execute(
                "update instance_presence set last_heartbeat_at = now() - interval '31 seconds';
                 update proxy_registrations set reported_at = now() - interval '31 seconds';",
            )
            .map_err(|error| error.to_string())?;
        let stale = lkjmc_store::status::snapshot(database.client_mut())
            .map_err(|error| error.to_string())?
            .instances
            .into_iter()
            .find(|row| row.id == "quartz-world")
            .ok_or("stale backend missing")?;
        let availability = crate::commands::instance_availability::evaluate(
            crate::commands::instance_availability::Input {
                kind: &stale.kind,
                desired_state: &stale.desired_state,
                process_healthy: stale.process_healthy,
                connect_port: stale
                    .registered_port
                    .map(i64::from)
                    .or(stale.configured_port),
                heartbeat_ready: stale.heartbeat_ready,
                heartbeat_age_seconds: stale.heartbeat_age_seconds,
                proxy_registration_desired: true,
                proxy_registered: stale.proxy_registered,
                proxy_failure_reason: stale.proxy_failure_reason.as_deref(),
                proxy_registration_age_seconds: stale.proxy_registration_age_seconds,
            },
        );
        assert!(!availability.joinable);
        assert_eq!(availability.ready, Some(false));
        assert_eq!(availability.join_disabled_reason, "heartbeat-stale");
        Ok(())
    }

    fn subject(surface: &str, instance_id: &str) -> AuthenticatedSubject {
        AuthenticatedSubject::credential(lkjmc_store::daemon_token::DaemonTokenRecord {
            credential_id: Uuid::nil(),
            surface: surface.into(),
            principal_kind: "instance".into(),
            principal_id: instance_id.into(),
            scopes: vec!["lkjmc.instance.heartbeat".into()],
            expires_at_micros: i64::MAX,
        })
    }

    fn empty_state() -> AppState {
        AppState::with_config_path(
            None,
            1,
            "/tmp/lkjmc-config".into(),
            "/tmp/lkjmc-log".into(),
            "/tmp/lkjmc-jars".into(),
            "/tmp/lkjmc-data".into(),
            None,
            None,
            None,
        )
    }

    fn call(state: AppState, request: Request<Body>) -> Result<Response, String> {
        std::thread::spawn(move || {
            let mut router = crate::transport::routes::router(state, true);
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|error| error.to_string())?;
            runtime.block_on(async {
                router
                    .call(request)
                    .await
                    .map_err(|error| error.to_string())
            })
        })
        .join()
        .map_err(|_| "heartbeat route worker panicked".to_string())?
    }

    fn request(token: &str, body: Body) -> Result<Request<Body>, String> {
        Request::builder()
            .method("POST")
            .uri("/plugin/v1/heartbeat")
            .header("authorization", format!("bEaReR {token}"))
            .body(body)
            .map_err(|error| error.to_string())
    }

    fn velocity_body() -> Body {
        Body::from(
            json!({
                "registrations": [
                    {
                        "instanceId": "ember-realm",
                        "connectHost": "127.0.0.1",
                        "connectPort": 25567,
                        "registered": true
                    },
                    {
                        "instanceId": "quartz-world",
                        "connectHost": "127.0.0.1",
                        "connectPort": 25566,
                        "registered": true
                    }
                ]
            })
            .to_string(),
        )
    }

    fn state_for(database_url: &str) -> AppState {
        database_state(database_url.to_string())
    }

    fn database_state(database_url: String) -> AppState {
        AppState::with_config_path(
            Some(database_url),
            2,
            "/tmp/lkjmc-config".into(),
            "/tmp/lkjmc-log".into(),
            "/tmp/lkjmc-jars".into(),
            "/tmp/lkjmc-data".into(),
            None,
            None,
            None,
        )
    }
}
