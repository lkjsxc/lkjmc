#[allow(dead_code)]
mod support;

use std::sync::mpsc;
use std::time::Duration;

use lkjmc_store::{daemon_token, migrate, pool};
use uuid::Uuid;

#[test]
fn revocation_waits_for_an_inflight_revision_checked_authentication(
) -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    migrate::apply(database.client_mut())?;
    let credential_id = Uuid::new_v4();
    let token_hash = lkjmc_core::security::token_hash("race-token");
    daemon_token::insert(
        database.client_mut(),
        credential_id,
        &token_hash,
        "web",
        "operator",
        "operator-1",
        &["lkjmc.admin.admin".to_string()],
        3600,
    )?;

    let mut authenticator = pool::connect(database.url())?;
    let mut transaction = authenticator.transaction()?;
    daemon_token::lock_current_revision(&mut transaction)?;
    assert!(daemon_token::find_active(&mut transaction, &token_hash)?.is_some());
    let (sender, receiver) = mpsc::channel();
    let revoke_url = database.url().to_string();
    let revoker = std::thread::spawn(move || {
        let result = pool::connect(&revoke_url)
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
