#[allow(dead_code)]
mod support;

use lkjmc_store::{daemon_token, migrate, pool};
use std::env;
use uuid::Uuid;

#[test]
fn scoped_tokens_are_hashed_found_touched_and_revoked() -> Result<(), lkjmc_store::error::StoreError>
{
    let Some(database_url) = database_url() else {
        return Ok(());
    };
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
    let after_authentication = daemon_token::current_revision(&mut client)?;
    assert_eq!(daemon_token::touch_active(&mut client, &token_hash)?, 1);
    assert_eq!(
        daemon_token::current_revision(&mut client)?,
        after_authentication
    );
    assert_eq!(daemon_token::revoke(&mut client, credential_id)?, 1);
    assert!(daemon_token::current_revision(&mut client)? > after_authentication);
    assert!(daemon_token::find_active(&mut client, &token_hash)?.is_none());
    Ok(())
}

#[test]
fn migration_041_normalizes_legacy_daemon_token_rows() -> Result<(), lkjmc_store::error::StoreError>
{
    let Some(database_url) = database_url() else {
        return Ok(());
    };
    let mut client = pool::connect(&database_url)?;
    let _schema = support::prepare_isolated_schema(&mut client)?;
    client.batch_execute(
        "create table schema_migrations (
            version integer primary key, name text not null, checksum text,
            applied_at timestamptz not null default now()
        )",
    )?;
    for migration in migrate::migrations()
        .into_iter()
        .filter(|item| item.version <= 37)
    {
        client.batch_execute(migration.sql)?;
    }
    let credential_id = Uuid::new_v4();
    let token_hash = lkjmc_core::security::token_hash("pre-041");
    client.execute(
        "insert into daemon_tokens (credential_id, token_hash, surface) values ($1, $2, $3)",
        &[&credential_id, &token_hash, &"paper"],
    )?;
    for migration in migrate::migrations()
        .into_iter()
        .filter(|item| (38..=41).contains(&item.version))
    {
        client.batch_execute(migration.sql)?;
    }
    let row = client.query_one(
        "select surface, principal_kind, principal_id, scopes,
                expires_at <= now() + interval '24 hours'
         from daemon_tokens where credential_id = $1",
        &[&credential_id],
    )?;
    assert_eq!(row.get::<_, String>(0), "daemon");
    assert_eq!(row.get::<_, String>(1), "service");
    assert_eq!(row.get::<_, String>(2), format!("migrated-{credential_id}"));
    assert_eq!(row.get::<_, Vec<String>>(3), vec!["lkjmc.migrated.none"]);
    assert!(row.get::<_, bool>(4));
    Ok(())
}

fn database_url() -> Option<String> {
    match env::var("LKJMC_STORE_TEST_DATABASE_URL") {
        Ok(value) => Some(value),
        Err(_) => {
            eprintln!("skipped daemon-token store tests: LKJMC_STORE_TEST_DATABASE_URL is unset");
            None
        }
    }
}
