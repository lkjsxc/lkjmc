mod migration_list;

use std::collections::BTreeMap;

use postgres::Client;
use sha2::{Digest, Sha256};

use crate::error::StoreError;

const MIGRATION_LOCK: i64 = 7_526_470;

pub struct Migration {
    pub version: i32,
    pub name: &'static str,
    pub sql: &'static str,
}

pub fn migrations() -> Vec<Migration> {
    migration_list::migrations()
}

pub fn embedded_len() -> usize {
    migrations().len()
}

pub fn embedded_versions() -> Vec<i32> {
    migrations()
        .into_iter()
        .map(|migration| migration.version)
        .collect()
}

pub fn apply(client: &mut Client) -> Result<Vec<i32>, StoreError> {
    exclusive(client, |client| {
        ensure_table(client)?;
        let applied = verify_ledger(client)?;
        let mut inserted = Vec::new();
        for migration in migrations() {
            if applied.contains_key(&migration.version) {
                continue;
            }
            let mut transaction = client.transaction()?;
            transaction.batch_execute(migration.sql)?;
            transaction.execute(
                "insert into schema_migrations (version, name, checksum) values ($1, $2, $3)",
                &[
                    &migration.version,
                    &migration.name,
                    &checksum(migration.sql),
                ],
            )?;
            transaction.commit()?;
            inserted.push(migration.version);
        }
        Ok(inserted)
    })
}

pub fn applied_versions(client: &mut Client) -> Result<Vec<i32>, StoreError> {
    exclusive(client, |client| {
        ensure_table(client)?;
        Ok(verify_ledger(client)?.into_keys().collect())
    })
}

fn exclusive<T>(
    client: &mut Client,
    action: impl FnOnce(&mut Client) -> Result<T, StoreError>,
) -> Result<T, StoreError> {
    client.query_one("select pg_advisory_lock($1)", &[&MIGRATION_LOCK])?;
    let result = action(client);
    let unlock = client.query_one("select pg_advisory_unlock($1)", &[&MIGRATION_LOCK]);
    match (result, unlock) {
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error.into()),
        (Ok(value), Ok(_)) => Ok(value),
    }
}

fn verify_ledger(client: &mut Client) -> Result<BTreeMap<i32, String>, StoreError> {
    let known = migrations()
        .into_iter()
        .map(|migration| (migration.version, migration))
        .collect::<BTreeMap<_, _>>();
    let rows = client.query(
        "select version, name, checksum from schema_migrations order by version",
        &[],
    )?;
    let checksums_required = client
        .query_opt("select 1 from schema_migrations where version = 38", &[])?
        .is_some();
    let mut applied = BTreeMap::new();
    for row in rows {
        let version = row.get::<_, i32>(0);
        let name = row.get::<_, String>(1);
        let recorded = row.get::<_, Option<String>>(2);
        let migration = known
            .get(&version)
            .ok_or_else(|| StoreError::invalid_state(format!("unknown migration {version}")))?;
        if name != migration.name {
            return Err(StoreError::invalid_state(format!(
                "migration {version} name mismatch"
            )));
        }
        let expected = checksum(migration.sql);
        match recorded {
            Some(value) if value == expected => {}
            Some(_) => {
                return Err(StoreError::invalid_state(format!(
                    "migration {version} checksum mismatch"
                )))
            }
            None if checksums_required => {
                return Err(StoreError::invalid_state(format!(
                    "migration {version} checksum missing"
                )));
            }
            None => {
                client.execute(
                    "update schema_migrations set checksum = $2 where version = $1",
                    &[&version, &expected],
                )?;
            }
        }
        applied.insert(version, expected);
    }
    Ok(applied)
}

fn ensure_table(client: &mut Client) -> Result<(), StoreError> {
    client.batch_execute(
        "create table if not exists schema_migrations (
            version integer primary key,
            name text not null,
            checksum text,
            applied_at timestamptz not null default now()
        );
        alter table schema_migrations add column if not exists checksum text;",
    )?;
    Ok(())
}

fn checksum(sql: &str) -> String {
    format!("{:x}", Sha256::digest(sql.as_bytes()))
}

pub(crate) fn m(version: i32, name: &'static str, sql: &'static str) -> Migration {
    Migration { version, name, sql }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_migrations_have_unique_versions_and_checksums() {
        let migrations = migrations();
        let mut versions = migrations
            .iter()
            .map(|migration| migration.version)
            .collect::<Vec<_>>();
        versions.sort_unstable();
        versions.dedup();
        assert_eq!(versions.len(), migrations.len());
        assert!(migrations
            .iter()
            .all(|migration| !checksum(migration.sql).is_empty()));
    }
}
