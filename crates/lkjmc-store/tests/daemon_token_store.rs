#[allow(dead_code)]
mod support;

use lkjmc_store::{daemon_token, migrate};
use uuid::Uuid;

#[test]
fn scoped_tokens_are_hashed_found_touched_and_revoked() -> Result<(), lkjmc_store::error::StoreError>
{
    let Some(mut database) = database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    let credential_id = Uuid::new_v4();
    let revision = daemon_token::current_revision(client)?;
    let token_hash = lkjmc_core::security::token_hash("paper-token");
    daemon_token::insert(
        client,
        credential_id,
        &token_hash,
        "paper",
        "minecraft-player",
        "player-1",
        &["lkjmc.user.menu".to_string()],
        3600,
    )?;
    assert!(daemon_token::current_revision(client)? > revision);
    let token = daemon_token::find_active(client, &token_hash)?
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("token missing"))?;
    assert_eq!(token.surface, "paper");
    assert_eq!(token.scopes, vec!["lkjmc.user.menu".to_string()]);
    let after_authentication = daemon_token::current_revision(client)?;
    assert_eq!(daemon_token::touch_active(client, &token_hash)?, 1);
    assert_eq!(
        daemon_token::current_revision(client)?,
        after_authentication
    );
    assert_eq!(daemon_token::revoke(client, credential_id)?, 1);
    assert!(daemon_token::current_revision(client)? > after_authentication);
    assert!(daemon_token::find_active(client, &token_hash)?.is_none());
    Ok(())
}

#[test]
fn find_active_preserves_fractional_expiry_microseconds(
) -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut database) = database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    let credential_id = Uuid::new_v4();
    let token_hash = lkjmc_core::security::token_hash("fractional-expiry-token");
    daemon_token::insert(
        client,
        credential_id,
        &token_hash,
        "web",
        "operator",
        "operator-1",
        &["lkjmc.admin.admin".to_string()],
        3600,
    )?;
    client.execute(
        "update daemon_tokens set expires_at = date_trunc('second', clock_timestamp())
         + interval '10 seconds 900 milliseconds' where credential_id = $1",
        &[&credential_id],
    )?;
    let expected = client
        .query_one(
            "select floor(extract(epoch from expires_at) * 1000000)::bigint
         from daemon_tokens where credential_id = $1",
            &[&credential_id],
        )?
        .get::<_, i64>(0);
    let record = daemon_token::find_active(client, &token_hash)?
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("fractional token missing"))?;
    assert_eq!(record.expires_at_micros, expected);
    assert_eq!(expected.rem_euclid(1_000_000), 900_000);
    Ok(())
}

#[test]
fn migration_041_normalizes_legacy_daemon_token_rows() -> Result<(), lkjmc_store::error::StoreError>
{
    let Some(mut database) = database()? else {
        return Ok(());
    };
    let client = database.client_mut();
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

fn database() -> Result<Option<support::TestDatabase>, lkjmc_store::error::StoreError> {
    support::database()
}
