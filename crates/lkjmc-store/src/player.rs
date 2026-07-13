mod identity;
mod lease;
mod snapshot;

pub use identity::{ensure_identity, insert_identity};
pub use lease::{acquire_lease, LeaseRecord};
pub use snapshot::{latest_snapshot, snapshot_by_id, write_snapshot, NewSnapshot, SnapshotRecord};

use postgres::Client;
use uuid::Uuid;

use crate::error::StoreError;

pub fn get_identity_name(
    client: &mut Client,
    player_uuid: Uuid,
) -> Result<Option<String>, StoreError> {
    let row = client.query_opt(
        "select current_name from player_identities where player_uuid = $1",
        &[&player_uuid],
    )?;
    Ok(row.map(|row| row.get(0)))
}

pub fn snapshot_count(client: &mut Client, player_uuid: Uuid) -> Result<i64, StoreError> {
    let row = client.query_one(
        "select count(*)::bigint from player_profile_snapshots where player_uuid = $1",
        &[&player_uuid],
    )?;
    Ok(row.get(0))
}
