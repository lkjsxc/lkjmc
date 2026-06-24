use postgres::Client;

use crate::error::StoreError;

pub struct Migration {
    pub version: i32,
    pub name: &'static str,
    pub sql: &'static str,
}

pub fn migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            name: "core",
            sql: include_str!("../../../migrations/001-core.sql"),
        },
        Migration {
            version: 2,
            name: "instances",
            sql: include_str!("../../../migrations/002-instances.sql"),
        },
        Migration {
            version: 3,
            name: "jar-assets",
            sql: include_str!("../../../migrations/003-jar-assets.sql"),
        },
        Migration {
            version: 4,
            name: "player-profiles",
            sql: include_str!("../../../migrations/004-player-profiles.sql"),
        },
        Migration {
            version: 5,
            name: "audit-events",
            sql: include_str!("../../../migrations/005-audit-events.sql"),
        },
        Migration {
            version: 6,
            name: "ui-settings",
            sql: include_str!("../../../migrations/006-ui-settings.sql"),
        },
        Migration {
            version: 7,
            name: "party-invites",
            sql: include_str!("../../../migrations/007-party-invites.sql"),
        },
        Migration {
            version: 8,
            name: "shop",
            sql: include_str!("../../../migrations/008-shop.sql"),
        },
    ]
}

pub fn apply(client: &mut Client) -> Result<Vec<i32>, StoreError> {
    client.batch_execute(
        "create table if not exists schema_migrations (
            version integer primary key,
            name text not null,
            applied_at timestamptz not null default now()
        )",
    )?;
    let mut applied = Vec::new();
    for migration in migrations() {
        if is_applied(client, migration.version)? {
            continue;
        }
        let mut transaction = client.transaction()?;
        transaction.batch_execute(migration.sql)?;
        transaction.execute(
            "insert into schema_migrations (version, name) values ($1, $2)",
            &[&migration.version, &migration.name],
        )?;
        transaction.commit()?;
        applied.push(migration.version);
    }
    Ok(applied)
}

pub fn applied_versions(client: &mut Client) -> Result<Vec<i32>, StoreError> {
    client.batch_execute(
        "create table if not exists schema_migrations (
            version integer primary key,
            name text not null,
            applied_at timestamptz not null default now()
        )",
    )?;
    let rows = client.query(
        "select version from schema_migrations order by version",
        &[],
    )?;
    Ok(rows.into_iter().map(|row| row.get(0)).collect())
}

fn is_applied(client: &mut Client, version: i32) -> Result<bool, StoreError> {
    let row = client.query_opt(
        "select version from schema_migrations where version = $1",
        &[&version],
    )?;
    Ok(row.is_some())
}
