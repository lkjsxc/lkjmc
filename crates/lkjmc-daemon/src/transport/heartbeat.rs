use axum::body::{to_bytes, Body};
use axum::extract::{Extension, State};
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use tokio::time::timeout_at;

use crate::app::{AppState, BlockingError, RequestAdmission};
use crate::authz::AuthenticatedSubject;

const BODY_LIMIT: usize = 1;

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
    match timeout_at(
        admission.deadline(),
        to_bytes(request.into_body(), BODY_LIMIT),
    )
    .await
    {
        Ok(Ok(body)) if body.is_empty() => {}
        Ok(Ok(_)) | Ok(Err(_)) => {
            return error(StatusCode::BAD_REQUEST, "heartbeat.body_not_empty")
        }
        Err(_) => return error(StatusCode::REQUEST_TIMEOUT, "heartbeat.deadline_exceeded"),
    }

    let work = move || record(&state, &surface, &instance_id);
    match admission.run_blocking(work).await {
        Ok(Ok(())) => StatusCode::NO_CONTENT.into_response(),
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

fn record(state: &AppState, surface: &str, instance_id: &str) -> Result<(), HeartbeatError> {
    let mut client = state
        .request_database_connection()
        .map_err(HeartbeatError::Store)?;
    let mut transaction = client.transaction().map_err(store_error)?;
    let kind = transaction
        .query_opt("select kind from instances where id = $1", &[&instance_id])
        .map_err(store_error)?
        .map(|row| row.get::<_, String>(0))
        .ok_or(HeartbeatError::Denied)?;
    if !surface_matches_kind(surface, &kind) || (surface == "velocity" && instance_id != "proxy") {
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
        record_static_proxy_registrations(&mut transaction)?;
    }
    transaction.commit().map_err(store_error)?;
    Ok(())
}

fn record_static_proxy_registrations(
    transaction: &mut postgres::Transaction<'_>,
) -> Result<(), HeartbeatError> {
    let mut registrations = Vec::with_capacity(2);
    for id in ["hub", "survival"] {
        let row = transaction
            .query_opt(
                "select kind, desired_state, config from instances where id = $1",
                &[&id],
            )
            .map_err(store_error)?
            .ok_or(HeartbeatError::Denied)?;
        let kind: String = row.get(0);
        let desired_state: String = row.get(1);
        let config: serde_json::Value = row.get(2);
        let port = config
            .get("serverPort")
            .and_then(serde_json::Value::as_i64)
            .and_then(|value| i32::try_from(value).ok())
            .filter(|value| (1..=65_535).contains(value))
            .ok_or(HeartbeatError::Denied)?;
        if !matches!(kind.as_str(), "paper" | "folia" | "purpur") || desired_state != "running" {
            return Err(HeartbeatError::Denied);
        }
        registrations.push((id, port));
    }
    let reports = registrations
        .iter()
        .map(
            |(id, port)| lkjmc_store::proxy_registration::RegistrationReport {
                instance_id: id,
                connect_host: "127.0.0.1",
                connect_port: *port,
                registered: true,
                failure_reason: None,
            },
        )
        .collect::<Vec<_>>();
    lkjmc_store::proxy_registration::report_in(transaction, &reports).map_err(HeartbeatError::Store)
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
            Extension(subject("paper", "hub")),
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
            "hub",
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
            "hub",
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
            lkjmc_store::instance_presence::get(database.client_mut(), "hub")
                .map_err(|error| error.to_string())?
                .is_none()
        );
        Ok(())
    }

    #[test]
    fn scoped_heartbeat_is_empty_body_and_instance_bound() -> Result<(), String> {
        let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
            eprintln!("SKIP heartbeat integration: LKJMC_STORE_TEST_DATABASE_URL is unset");
            return Ok(());
        };
        let mut database = crate::test_database::migrate(&database_url)?;
        lkjmc_store::instance::insert(
            database.client_mut(),
            "hub",
            None,
            "folia",
            "running",
            &json!({"serverPort": 25566}),
        )
        .map_err(|error| error.to_string())?;
        lkjmc_store::instance::insert(
            database.client_mut(),
            "survival",
            None,
            "folia",
            "running",
            &json!({"serverPort": 25567}),
        )
        .map_err(|error| error.to_string())?;
        lkjmc_store::instance::insert(
            database.client_mut(),
            "proxy",
            None,
            "velocity",
            "running",
            &json!({"serverPort": 25591}),
        )
        .map_err(|error| error.to_string())?;
        let token = "MiXeD-Case_heartbeat-token";
        let survival_token = "distinct-survival-heartbeat-token";
        let proxy_token = "distinct-velocity-heartbeat-token";
        let wrong_scope = "heartbeat-token-with-sync-only";
        lkjmc_store::daemon_token::insert(
            database.client_mut(),
            Uuid::new_v4(),
            &lkjmc_core::security::token_hash(token),
            "paper",
            "instance",
            "hub",
            &["lkjmc.instance.heartbeat".to_string()],
            3600,
        )
        .map_err(|error| error.to_string())?;
        lkjmc_store::daemon_token::insert(
            database.client_mut(),
            Uuid::new_v4(),
            &lkjmc_core::security::token_hash(survival_token),
            "paper",
            "instance",
            "survival",
            &["lkjmc.instance.heartbeat".to_string()],
            3600,
        )
        .map_err(|error| error.to_string())?;
        lkjmc_store::daemon_token::insert(
            database.client_mut(),
            Uuid::new_v4(),
            &lkjmc_core::security::token_hash(proxy_token),
            "velocity",
            "instance",
            "proxy",
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
            "hub",
            &["lkjmc.sync.read".to_string()],
            3600,
        )
        .map_err(|error| error.to_string())?;
        for (id, pid) in [("hub", 1001), ("survival", 1002)] {
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
            lkjmc_store::instance_presence::get(database.client_mut(), "hub")
                .map_err(|error| error.to_string())?
                .is_none()
        );

        let response = call(state, request(token, Body::empty())?)?;
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let presence = lkjmc_store::instance_presence::get(database.client_mut(), "hub")
            .map_err(|error| error.to_string())?
            .ok_or("heartbeat presence missing")?;
        assert!(presence.ready);
        assert_eq!(presence.player_count, None);
        assert_eq!(presence.max_players, None);
        assert!(presence.heartbeat_age_seconds.is_some_and(|age| age <= 1));
        let survival_response = call(
            state_for(database.url()),
            request(survival_token, Body::empty())?,
        )?;
        assert_eq!(survival_response.status(), StatusCode::NO_CONTENT);

        lkjmc_store::instance::update_config(
            database.client_mut(),
            "survival",
            &json!({"serverPort": 0}),
        )
        .map_err(|error| error.to_string())?;
        let rolled_back = call(
            state_for(database.url()),
            request(proxy_token, Body::empty())?,
        )?;
        assert_eq!(rolled_back.status(), StatusCode::FORBIDDEN);
        assert!(
            lkjmc_store::instance_presence::get(database.client_mut(), "proxy")
                .map_err(|error| error.to_string())?
                .is_none()
        );
        assert!(
            lkjmc_store::proxy_registration::get(database.client_mut(), "hub")
                .map_err(|error| error.to_string())?
                .is_none()
        );
        lkjmc_store::instance::update_config(
            database.client_mut(),
            "survival",
            &json!({"serverPort": 25567}),
        )
        .map_err(|error| error.to_string())?;

        let proxy_response = call(
            state_for(database.url()),
            request(proxy_token, Body::empty())?,
        )?;
        assert_eq!(proxy_response.status(), StatusCode::NO_CONTENT);
        for (id, port) in [("hub", 25566), ("survival", 25567)] {
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
            .find(|row| row.id == "hub")
            .ok_or("stale hub missing")?;
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
