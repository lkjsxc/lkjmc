use postgres::{Client, Row};
use uuid::Uuid;

use lkjmc_core::profile_validation::canonical_profile;

use crate::error::StoreError;

use super::SnapshotRecord;

pub fn latest_snapshot(
    client: &mut Client,
    player: Uuid,
    scope: &str,
) -> Result<Option<SnapshotRecord>, StoreError> {
    query_snapshot(client, player, scope, None)
}

pub fn snapshot_by_id(
    client: &mut Client,
    id: Uuid,
    player: Uuid,
    scope: &str,
) -> Result<Option<SnapshotRecord>, StoreError> {
    query_snapshot(client, player, scope, Some(id))
}

fn query_snapshot(
    client: &mut Client,
    player: Uuid,
    scope: &str,
    id: Option<Uuid>,
) -> Result<Option<SnapshotRecord>, StoreError> {
    let row = client.query_opt(
        "select id,player_uuid,scope,revision,session_revision,lease_fence,
         correlation_id,canonical_json,sha256 from player_profile_snapshots
         where player_uuid = $1 and scope = $2 and ($3::uuid is null or id = $3)
         order by revision desc limit 1",
        &[&player, &scope, &id],
    )?;
    row.map(|row| from_row(&row, false)).transpose()
}

pub(super) fn from_row(row: &Row, replay: bool) -> Result<SnapshotRecord, StoreError> {
    let json: Vec<u8> = row.get(7);
    let canonical = canonical_profile(&json).map_err(StoreError::invalid_state)?;
    let stored_sha: String = row.get(8);
    if canonical.json != json || canonical.sha256 != stored_sha {
        return Err(StoreError::invalid_state(
            "stored profile is not canonical or intact",
        ));
    }
    Ok(SnapshotRecord {
        id: row.get(0),
        player_uuid: row.get(1),
        scope: row.get(2),
        revision: row.get(3),
        session_revision: row.get(4),
        lease_fence: row.get(5),
        correlation_id: row.get(6),
        envelope: canonical.envelope,
        canonical_json: json,
        sha256: stored_sha,
        replay,
    })
}
