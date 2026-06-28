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
        Migration {
            version: 9,
            name: "player-teleports",
            sql: include_str!("../../../migrations/009-player-teleports.sql"),
        },
        Migration {
            version: 10,
            name: "player-mail",
            sql: include_str!("../../../migrations/010-player-mail.sql"),
        },
        Migration {
            version: 11,
            name: "player-reports",
            sql: include_str!("../../../migrations/011-player-reports.sql"),
        },
        Migration {
            version: 12,
            name: "player-punishments",
            sql: include_str!("../../../migrations/012-player-punishments.sql"),
        },
        Migration {
            version: 13,
            name: "daily-rewards",
            sql: include_str!("../../../migrations/013-daily-rewards.sql"),
        },
        Migration {
            version: 14,
            name: "player-warnings",
            sql: include_str!("../../../migrations/014-player-warnings.sql"),
        },
        Migration {
            version: 15,
            name: "announcements",
            sql: include_str!("../../../migrations/015-announcements.sql"),
        },
        Migration {
            version: 16,
            name: "player-kits",
            sql: include_str!("../../../migrations/016-player-kits.sql"),
        },
        Migration {
            version: 17,
            name: "player-notes",
            sql: include_str!("../../../migrations/017-player-notes.sql"),
        },
        Migration {
            version: 18,
            name: "vote-links",
            sql: include_str!("../../../migrations/018-vote-links.sql"),
        },
        Migration {
            version: 19,
            name: "vote-rewards",
            sql: include_str!("../../../migrations/019-vote-rewards.sql"),
        },
        Migration {
            version: 20,
            name: "chat-mutes",
            sql: include_str!("../../../migrations/020-chat-mutes.sql"),
        },
        Migration {
            version: 21,
            name: "claims",
            sql: include_str!("../../../migrations/021-claims.sql"),
        },
        Migration {
            version: 22,
            name: "assets-bootstrap",
            sql: include_str!("../../../migrations/022-assets-bootstrap.sql"),
        },
        Migration {
            version: 23,
            name: "presence-autosuspend",
            sql: include_str!("../../../migrations/023-presence-autosuspend.sql"),
        },
        Migration {
            version: 24,
            name: "temporary-adventures",
            sql: include_str!("../../../migrations/024-temporary-adventures.sql"),
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
