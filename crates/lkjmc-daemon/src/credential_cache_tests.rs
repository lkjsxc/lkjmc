use super::*;
use uuid::Uuid;

#[test]
fn revision_change_drops_cached_credential() -> Result<(), String> {
    let cache = CredentialCache::default();
    cache.store("hash".into(), 1, record(60)).map_err(failed)?;
    assert!(cache.cached("hash", 1).map_err(failed)?.is_some());
    assert!(cache.cached("hash", 2).map_err(failed)?.is_none());
    Ok(())
}

#[test]
fn capacity_evicts_lowest_hash_deterministically() -> Result<(), String> {
    let cache = CredentialCache::default();
    for index in 0..MAX_CREDENTIALS {
        cache
            .store(format!("{index:03}"), 1, record(60))
            .map_err(failed)?;
    }
    cache.store("zzz".into(), 1, record(60)).map_err(failed)?;
    assert!(cache.cached("000", 1).map_err(failed)?.is_none());
    assert!(cache.cached("zzz", 1).map_err(failed)?.is_some());
    Ok(())
}

#[test]
fn expired_credential_is_not_returned() -> Result<(), String> {
    let cache = CredentialCache::default();
    cache.store("hash".into(), 1, record(-1)).map_err(failed)?;
    assert!(cache.cached("hash", 1).map_err(failed)?.is_none());
    Ok(())
}

#[test]
fn fractional_expiry_is_not_rounded_up_in_cache() {
    let mut entries = std::collections::BTreeMap::new();
    entries.insert("hash".into(), record_at(1_900_000));
    expire_at(&mut entries, 1_950_000);
    assert!(entries.is_empty());
}

#[test]
fn database_authentication_uses_a_cache_hit_without_bumping_revision() -> Result<(), String> {
    let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        eprintln!("SKIP credential cache database test: LKJMC_STORE_TEST_DATABASE_URL is unset");
        return Ok(());
    };
    let mut database = crate::test_database::reset_and_migrate(&database_url)?;
    let token = "credential-cache-canary";
    let token_hash = lkjmc_core::security::token_hash(token);
    let credential_id = Uuid::new_v4();
    lkjmc_store::daemon_token::insert(
        database.client_mut(),
        credential_id,
        &token_hash,
        "web",
        "operator",
        "operator-1",
        &["lkjmc.admin.admin".to_string()],
        3600,
    )
    .map_err(|error| error.to_string())?;
    let expected_revision = lkjmc_store::daemon_token::current_revision(database.client_mut())
        .map_err(|error| error.to_string())?;
    let cache = CredentialCache::default();
    let mut client =
        lkjmc_store::pool::connect(&database_url).map_err(|error| error.to_string())?;
    assert!(cache
        .authenticate(&mut client, token)
        .map_err(failed)?
        .is_some());
    let first_touch = touched_at(&mut client, credential_id)?;
    assert!(cache
        .authenticate(&mut client, token)
        .map_err(failed)?
        .is_some());
    assert_eq!(touched_at(&mut client, credential_id)?, first_touch);
    assert_eq!(
        lkjmc_store::daemon_token::current_revision(&mut client)
            .map_err(|error| error.to_string())?,
        expected_revision
    );
    lkjmc_store::daemon_token::revoke(database.client_mut(), credential_id)
        .map_err(|error| error.to_string())?;
    assert!(cache
        .authenticate(&mut client, token)
        .map_err(failed)?
        .is_none());
    Ok(())
}

#[test]
fn lock_timeout_remains_deadline_through_cache() -> Result<(), String> {
    let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let mut database = crate::test_database::migrate(&database_url)?;
    let mut transaction = database
        .client_mut()
        .transaction()
        .map_err(|error| error.to_string())?;
    transaction
        .query_one(
            "select revision from daemon_token_revision where singleton = true for update",
            &[],
        )
        .map_err(|error| error.to_string())?;
    let mut client =
        lkjmc_store::pool::connect(&database_url).map_err(|error| error.to_string())?;
    client
        .batch_execute("set lock_timeout = '10ms'; set statement_timeout = '100ms'")
        .map_err(|error| error.to_string())?;
    let error = match CredentialCache::default().authenticate(&mut client, "cache-lock-timeout") {
        Err(error) => error,
        Ok(_) => return Err("credential cache did not preserve the lock timeout".into()),
    };
    assert!(error.is_deadline());
    drop(transaction);
    Ok(())
}

#[test]
fn unavailable_database_denies_even_a_cached_credential() -> Result<(), String> {
    let state = crate::app::AppState::with_config_path(
        Some("not-a-database-url".into()),
        1,
        "/config".into(),
        "/log".into(),
        "/jars".into(),
        "/data".into(),
        None,
        None,
        None,
    );
    let hash = lkjmc_core::security::token_hash("security-canary");
    state
        .credential_cache
        .store(hash, 1, record(60))
        .map_err(failed)?;
    assert!(state.authenticate_credential("security-canary").is_err());
    Ok(())
}

fn touched_at(client: &mut postgres::Client, credential_id: Uuid) -> Result<String, String> {
    client
        .query_one(
            "select last_used_at::text from daemon_tokens where credential_id = $1",
            &[&credential_id],
        )
        .map(|row| row.get(0))
        .map_err(|error| error.to_string())
}

fn record(offset_seconds: i64) -> DaemonTokenRecord {
    record_at(unix_micros() + offset_seconds * 1_000_000)
}

fn record_at(expires_at_micros: i64) -> DaemonTokenRecord {
    DaemonTokenRecord {
        credential_id: Uuid::nil(),
        surface: "web".into(),
        principal_kind: "operator".into(),
        principal_id: "operator-1".into(),
        scopes: vec!["lkjmc.admin.admin".into()],
        expires_at_micros,
    }
}

fn failed(error: lkjmc_store::error::StoreError) -> String {
    format!("cache operation failed: {error}")
}
