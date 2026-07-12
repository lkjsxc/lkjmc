#[allow(dead_code)]
mod support;

use std::sync::mpsc;
use std::time::Duration;

use lkjmc_store::{daemon_token, migrate, pool};
use uuid::Uuid;

#[test]
fn revocation_waits_for_an_inflight_revision_checked_authentication(
) -> Result<(), lkjmc_store::error::StoreError> {
    let Some(database_url) = database_url() else {
        return Ok(());
    };
    let mut setup = pool::connect(&database_url)?;
    let schema = support::prepare_isolated_schema(&mut setup)?;
    migrate::apply(&mut setup)?;
    let credential_id = Uuid::new_v4();
    let token_hash = lkjmc_core::security::token_hash("race-token");
    daemon_token::insert(
        &mut setup,
        credential_id,
        &token_hash,
        "web",
        "operator",
        "operator-1",
        &["lkjmc.admin.admin".to_string()],
        3600,
    )?;

    let mut authenticator = connection(&database_url, &schema)?;
    let mut transaction = authenticator.transaction()?;
    daemon_token::lock_current_revision(&mut transaction)?;
    assert!(daemon_token::find_active(&mut transaction, &token_hash)?.is_some());
    let (sender, receiver) = mpsc::channel();
    let revoke_url = database_url.clone();
    let revoke_schema = schema.clone();
    let revoker = std::thread::spawn(move || {
        let result = connection(&revoke_url, &revoke_schema)
            .and_then(|mut client| daemon_token::revoke(&mut client, credential_id));
        let _ = sender.send(result);
    });
    assert!(receiver.recv_timeout(Duration::from_millis(100)).is_err());
    transaction.commit()?;
    assert_eq!(
        receiver
            .recv_timeout(Duration::from_secs(5))
            .map_err(|error| lkjmc_store::error::StoreError::invalid_state(error.to_string()))??,
        1
    );
    revoker
        .join()
        .map_err(|_| lkjmc_store::error::StoreError::invalid_state("revoker thread panicked"))?;
    assert!(daemon_token::find_active(&mut authenticator, &token_hash)?.is_none());
    Ok(())
}

fn connection(
    database_url: &str,
    schema: &str,
) -> Result<postgres::Client, lkjmc_store::error::StoreError> {
    let mut client = pool::connect(database_url)?;
    client.batch_execute(&format!("set search_path to {schema}, public"))?;
    Ok(client)
}

fn database_url() -> Option<String> {
    match std::env::var("LKJMC_STORE_TEST_DATABASE_URL") {
        Ok(value) => Some(value),
        Err(_) => {
            eprintln!("SKIP daemon-token revision test: LKJMC_STORE_TEST_DATABASE_URL is unset");
            None
        }
    }
}
