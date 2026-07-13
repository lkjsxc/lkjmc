#[allow(dead_code)]
mod support;

use std::sync::{Arc, Barrier};
use std::thread;

use lkjmc_store::{migrate, pool};

#[test]
fn migration_checksum_rejects_tampering() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    client.execute(
        "update schema_migrations set checksum = 'wrong' where version = 1",
        &[],
    )?;
    let error = migrate::apply(client)
        .err()
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("tampering passed"))?;
    assert!(error.to_string().contains("checksum mismatch"));
    Ok(())
}

#[test]
fn migration_checksum_backfills_once_then_rejects_null(
) -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    client.execute("delete from schema_migrations where version = 38", &[])?;
    client.execute(
        "update schema_migrations set checksum = null where version = 1",
        &[],
    )?;
    assert_eq!(migrate::apply(client)?, vec![38]);
    client.execute(
        "update schema_migrations set checksum = null where version = 1",
        &[],
    )?;
    let error = migrate::apply(client)
        .err()
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("null checksum passed"))?;
    assert!(error.to_string().contains("checksum missing"));
    Ok(())
}

#[test]
fn concurrent_migrations_serialize_to_one_writer() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(database) = support::database()? else {
        return Ok(());
    };
    let url = database.url().to_string();
    let barrier = Arc::new(Barrier::new(3));
    let mut workers = Vec::new();
    for _ in 0..2 {
        let url = url.clone();
        let barrier = Arc::clone(&barrier);
        workers.push(thread::spawn(move || -> Result<Vec<i32>, String> {
            let mut client = pool::connect(&url).map_err(|error| error.to_string())?;
            barrier.wait();
            migrate::apply(&mut client).map_err(|error| error.to_string())
        }));
    }
    barrier.wait();
    let mut counts = Vec::new();
    for worker in workers {
        let applied = worker
            .join()
            .map_err(|_| {
                lkjmc_store::error::StoreError::invalid_state("migration worker panicked")
            })?
            .map_err(lkjmc_store::error::StoreError::invalid_state)?;
        counts.push(applied.len());
    }
    counts.sort_unstable();
    assert_eq!(counts, vec![0, migrate::embedded_len()]);
    Ok(())
}

#[test]
fn deadline_connection_uses_its_supplied_budget() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(database) = support::database()? else {
        return Ok(());
    };
    let url = database.url().to_string();
    let mut client = pool::connect_with_deadline(&url, std::time::Duration::from_millis(321))?;
    assert_eq!(setting(&mut client, "statement_timeout")?, "321ms");
    assert_eq!(setting(&mut client, "lock_timeout")?, "321ms");
    let database = pool::build(&url, 1, std::time::Duration::from_millis(321))?;
    let mut pooled = database
        .get()
        .map_err(|error| lkjmc_store::error::StoreError::invalid_state(error.to_string()))?;
    assert_eq!(setting(&mut pooled, "statement_timeout")?, "321ms");
    assert_eq!(setting(&mut pooled, "lock_timeout")?, "321ms");
    Ok(())
}

fn setting(
    client: &mut postgres::Client,
    name: &str,
) -> Result<String, lkjmc_store::error::StoreError> {
    Ok(client.query_one(&format!("show {name}"), &[])?.get(0))
}
