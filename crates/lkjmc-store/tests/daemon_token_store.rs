#[allow(dead_code)]
mod support;

use lkjmc_store::{daemon_token, migrate, pool};
use std::env;
use uuid::Uuid;

#[test]
#[ignore = "requires LKJMC_STORE_TEST_DATABASE_URL"]
fn scoped_tokens_are_hashed_found_and_revoked() -> Result<(), lkjmc_store::error::StoreError> {
    let database_url = env::var("LKJMC_STORE_TEST_DATABASE_URL")
        .map_err(|_| lkjmc_store::error::StoreError::invalid_state("database URL is required"))?;
    let mut client = pool::connect(&database_url)?;
    let _schema = support::prepare_isolated_schema(&mut client)?;
    migrate::apply(&mut client)?;
    let credential_id = Uuid::new_v4();
    let revision = daemon_token::current_revision(&mut client)?;
    let token_hash = lkjmc_core::security::token_hash("paper-token");
    daemon_token::insert(
        &mut client,
        credential_id,
        &token_hash,
        "paper",
        "minecraft-player",
        "player-1",
        &["lkjmc.user.menu".to_string()],
        3600,
    )?;
    assert!(daemon_token::current_revision(&mut client)? > revision);
    let token = daemon_token::find_active(&mut client, &token_hash)?
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("token missing"))?;
    assert_eq!(token.surface, "paper");
    assert_eq!(token.scopes, vec!["lkjmc.user.menu".to_string()]);
    let after_insert = daemon_token::current_revision(&mut client)?;
    assert_eq!(daemon_token::revoke(&mut client, credential_id)?, 1);
    assert!(daemon_token::current_revision(&mut client)? > after_insert);
    assert!(daemon_token::find_active(&mut client, &token_hash)?.is_none());
    Ok(())
}

#[test]
#[ignore = "requires LKJMC_STORE_TEST_DATABASE_URL"]
fn migration_038_constrains_existing_token_rows() -> Result<(), lkjmc_store::error::StoreError> {
    let database_url = env::var("LKJMC_STORE_TEST_DATABASE_URL")
        .map_err(|_| lkjmc_store::error::StoreError::invalid_state("database URL is required"))?;
    let mut client = pool::connect(&database_url)?;
    let _schema = support::prepare_isolated_schema(&mut client)?;
    for migration in migrate::migrations()
        .into_iter()
        .filter(|item| item.version <= 37)
    {
        client.batch_execute(migration.sql)?;
    }
    let credential_id = Uuid::new_v4();
    client.execute(
        "insert into daemon_tokens (credential_id, token_hash, surface) values ($1, $2, $3)",
        &[
            &credential_id,
            &lkjmc_core::security::token_hash("pre-038"),
            &"paper",
        ],
    )?;
    let migration = migrate::migrations()
        .into_iter()
        .find(|item| item.version == 38)
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("migration 038 missing"))?;
    client.batch_execute(migration.sql)?;
    let row = client.query_one(
        "select surface, principal_kind, principal_id, scopes, expires_at <= now() + interval '24 hours' from daemon_tokens where credential_id = $1",
        &[&credential_id],
    )?;
    assert_eq!(row.get::<_, String>(0), "daemon");
    assert_eq!(row.get::<_, String>(1), "service");
    assert_eq!(row.get::<_, String>(2), format!("migrated-{credential_id}"));
    assert_eq!(row.get::<_, Vec<String>>(3), vec!["lkjmc.migrated.none"]);
    assert!(row.get::<_, bool>(4));
    Ok(())
}
