use postgres::Client;

use crate::error::StoreError;

pub struct Migration {
    pub version: i32,
    pub name: &'static str,
    pub sql: &'static str,
}

#[rustfmt::skip]
pub fn migrations() -> Vec<Migration> {
    vec![
        m(1, "core", include_str!("../../../migrations/001-core.sql")),
        m(2, "instances", include_str!("../../../migrations/002-instances.sql")),
        m(3, "jar-assets", include_str!("../../../migrations/003-jar-assets.sql")),
        m(4, "player-profiles", include_str!("../../../migrations/004-player-profiles.sql")),
        m(5, "audit-events", include_str!("../../../migrations/005-audit-events.sql")),
        m(6, "ui-settings", include_str!("../../../migrations/006-ui-settings.sql")),
        m(7, "party-invites", include_str!("../../../migrations/007-party-invites.sql")),
        m(8, "shop", include_str!("../../../migrations/008-shop.sql")),
        m(9, "player-teleports", include_str!("../../../migrations/009-player-teleports.sql")),
        m(10, "player-mail", include_str!("../../../migrations/010-player-mail.sql")),
        m(11, "player-reports", include_str!("../../../migrations/011-player-reports.sql")),
        m(12, "player-punishments", include_str!("../../../migrations/012-player-punishments.sql")),
        m(13, "daily-rewards", include_str!("../../../migrations/013-daily-rewards.sql")),
        m(14, "player-warnings", include_str!("../../../migrations/014-player-warnings.sql")),
        m(15, "announcements", include_str!("../../../migrations/015-announcements.sql")),
        m(16, "player-kits", include_str!("../../../migrations/016-player-kits.sql")),
        m(17, "player-notes", include_str!("../../../migrations/017-player-notes.sql")),
        m(18, "vote-links", include_str!("../../../migrations/018-vote-links.sql")),
        m(19, "vote-rewards", include_str!("../../../migrations/019-vote-rewards.sql")),
        m(20, "chat-mutes", include_str!("../../../migrations/020-chat-mutes.sql")),
        m(21, "claims", include_str!("../../../migrations/021-claims.sql")),
        m(22, "assets-bootstrap", include_str!("../../../migrations/022-assets-bootstrap.sql")),
        m(23, "presence-autosuspend", include_str!("../../../migrations/023-presence-autosuspend.sql")),
        m(24, "temporary-adventures", include_str!("../../../migrations/024-temporary-adventures.sql")),
        m(25, "temporary-transfer-intents", include_str!("../../../migrations/025-temporary-transfer-intents.sql")),
        m(26, "wake-join-queue", include_str!("../../../migrations/026-wake-join-queue.sql")),
        m(27, "economy-exchange", include_str!("../../../migrations/027-economy-exchange.sql")),
        m(28, "admin-rbac", include_str!("../../../migrations/028-admin-rbac.sql")),
        m(29, "wake-join-controls", include_str!("../../../migrations/029-wake-join-controls.sql")),
        m(30, "achievement-reward-claims", include_str!("../../../migrations/030-achievement-reward-claims.sql")),
        m(31, "random-teleports", include_str!("../../../migrations/031-random-teleports.sql")),
        m(32, "discord-account-links", include_str!("../../../migrations/032-discord-account-links.sql")),
        m(33, "link-codes", include_str!("../../../migrations/033-link-codes.sql")),
    ]
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
    ensure_table(client)?;
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
    ensure_table(client)?;
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

fn ensure_table(client: &mut Client) -> Result<(), StoreError> {
    client.batch_execute(
        "create table if not exists schema_migrations (
            version integer primary key,
            name text not null,
            applied_at timestamptz not null default now()
        )",
    )?;
    Ok(())
}

fn m(version: i32, name: &'static str, sql: &'static str) -> Migration {
    Migration { version, name, sql }
}
