#[allow(dead_code)]
mod support;

use lkjmc_store::{daemon_token, migrate, pool};
use std::env;
use uuid::Uuid;

#[test]
fn scoped_tokens_are_hashed_found_and_revoked() -> Result<(), lkjmc_store::error::StoreError> {
    let database_url = match env::var("LKJMC_STORE_TEST_DATABASE_URL") {
        Ok(value) => value,
        Err(_) => return Ok(()),
    };
    let mut client = pool::connect(&database_url)?;
    let _schema = support::prepare_isolated_schema(&mut client)?;
    migrate::apply(&mut client)?;
    let credential_id = Uuid::new_v4();
    let token_hash = lkjmc_core::security::token_hash("paper-token");
    daemon_token::insert(
        &mut client,
        credential_id,
        &token_hash,
        "paper",
        &["lkjmc.user.menu".to_string()],
    )?;
    let token = daemon_token::find_active(&mut client, &token_hash)?
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("token missing"))?;
    assert_eq!(token.surface, "paper");
    assert_eq!(token.scopes, vec!["lkjmc.user.menu".to_string()]);
    assert_eq!(daemon_token::revoke(&mut client, credential_id)?, 1);
    assert!(daemon_token::find_active(&mut client, &token_hash)?.is_none());
    Ok(())
}
