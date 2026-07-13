use std::time::Duration;

use axum::body::Bytes;
use axum::extract::{Extension, State};

use super::*;

#[test]
fn timeout_outcome_pass() -> Result<(), String> {
    let Some(url) = database_url() else {
        return Ok(());
    };
    let mut database = crate::test_database::migrate(&url)?;
    let database_url = database.url().to_string();
    let player = uuid::Uuid::parse_str(PLAYER).map_err(|error| error.to_string())?;
    lkjmc_store::player_settings::set_language_for_identity(
        database.client_mut(),
        player,
        "Repeat",
        "en",
    )
    .map_err(|error| error.to_string())?;
    let mut blocker = database
        .client_mut()
        .transaction()
        .map_err(|error| error.to_string())?;
    blocker
        .query_one(
            "select player_uuid from player_identities where player_uuid = $1 for update",
            &[&player],
        )
        .map_err(|error| error.to_string())?;
    blocker
        .query_one(
            "select player_uuid from player_settings where player_uuid = $1 for update",
            &[&player],
        )
        .map_err(|error| error.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    let envelope = request(&id, "ja")?;
    let state = crate::app::Admission::with_test_deadline(Duration::from_millis(300), || {
        state(database_url)
    });
    let admission = state
        .admit_request()
        .ok_or("request admission unavailable")?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    let response = runtime.block_on(async {
        let response = crate::transport::command::handle(
            State(state.clone()),
            Some(Extension(crate::authz::AuthenticatedSubject::internal())),
            Some(Extension(admission)),
            Bytes::from(serde_json::to_vec(&envelope).map_err(|error| error.to_string())?),
        )
        .await;
        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .map_err(|error| error.to_string())?;
        state.wait_for_admitted_work().await?;
        serde_json::from_slice::<CommandResponse>(&body).map_err(|error| error.to_string())
    })?;
    drop(blocker);
    assert_eq!(response.request_id.as_str(), id);
    assert!(!response.ok);
    assert_eq!(error_code(&response), "command.deadline_exceeded");
    let outcome = lkjmc_store::command::lookup(database.client_mut(), &id)
        .map_err(|error| error.to_string())?
        .ok_or("timed-out request has no durable outcome")?;
    assert_eq!(outcome.result, "cancelled");
    assert_eq!(outcome.response.request_id.as_str(), id);
    assert_eq!(error_code(&outcome.response), "command.deadline_exceeded");
    let running: i64 = database
        .client_mut()
        .query_one(
            "select count(*) from commands where result = 'requested'",
            &[],
        )
        .map_err(|error| error.to_string())?
        .get(0);
    assert_eq!(running, 0);
    Ok(())
}
