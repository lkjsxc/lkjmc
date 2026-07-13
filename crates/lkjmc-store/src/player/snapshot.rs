mod query;

use query::from_row;
pub use query::{latest_snapshot, snapshot_by_id};

use postgres::Client;
use uuid::Uuid;

use lkjmc_core::profile_envelope::ProfileEnvelope;
use lkjmc_core::profile_validation::canonical_profile;

use crate::data_workflows;
use crate::error::StoreError;

#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotRecord {
    pub id: Uuid,
    pub player_uuid: Uuid,
    pub scope: String,
    pub revision: i64,
    pub session_revision: i64,
    pub lease_fence: i64,
    pub correlation_id: Uuid,
    pub envelope: ProfileEnvelope,
    pub canonical_json: Vec<u8>,
    pub sha256: String,
    pub replay: bool,
}

pub struct NewSnapshot<'a> {
    pub id: Uuid,
    pub player_uuid: Uuid,
    pub scope: &'a str,
    pub session_id: Uuid,
    pub expected_session_revision: i64,
    pub expected_lease_fence: i64,
    pub expected_snapshot_revision: i64,
    pub correlation_id: Uuid,
    pub source_instance: &'a str,
    pub profile_json: &'a [u8],
}

pub fn write_snapshot(
    client: &mut Client,
    new: NewSnapshot<'_>,
) -> Result<SnapshotRecord, StoreError> {
    if new.scope.is_empty()
        || new.source_instance.is_empty()
        || new.expected_session_revision < 1
        || new.expected_lease_fence < 1
        || new.expected_snapshot_revision < 0
    {
        return Err(StoreError::invalid_state("invalid profile write boundary"));
    }
    let canonical = canonical_profile(new.profile_json).map_err(StoreError::invalid_state)?;
    let envelope_value = serde_json::to_value(&canonical.envelope)
        .map_err(|error| StoreError::invalid_state(error.to_string()))?;
    let mut tx = client.transaction()?;
    if let Some(row) = tx.query_opt(
        "select id, player_uuid, scope, revision, session_revision, lease_fence,
         correlation_id, canonical_json, sha256, session_id, expected_snapshot_revision,
         source_instance from player_profile_snapshots where correlation_id = $1 for update",
        &[&new.correlation_id],
    )? {
        let same = row.get::<_, Uuid>(0) == new.id
            && row.get::<_, Uuid>(1) == new.player_uuid
            && row.get::<_, String>(2) == new.scope
            && row.get::<_, i64>(4) == new.expected_session_revision
            && row.get::<_, i64>(5) == new.expected_lease_fence
            && row.get::<_, Uuid>(9) == new.session_id
            && row.get::<_, i64>(10) == new.expected_snapshot_revision
            && row.get::<_, String>(11) == new.source_instance
            && row.get::<_, Vec<u8>>(7) == canonical.json;
        if !same {
            return Err(StoreError::invalid_state("changed profile replay"));
        }
        let result = from_row(&row, true)?;
        tx.commit()?;
        return Ok(result);
    }
    let session = tx
        .query_opt(
            "select revision from player_sessions where id = $1 and player_uuid = $2
         and left_at is null for update",
            &[&new.session_id, &new.player_uuid],
        )?
        .ok_or_else(|| StoreError::invalid_state("active profile session missing"))?;
    if session.get::<_, i64>(0) != new.expected_session_revision {
        return Err(StoreError::invalid_state("stale profile session revision"));
    }
    let lease = tx
        .query_opt(
            "select fence, holder from player_profile_leases where player_uuid = $1
         and scope = $2 and expires_at > now() for update",
            &[&new.player_uuid, &new.scope],
        )?
        .ok_or_else(|| StoreError::invalid_state("active profile lease missing"))?;
    if lease.get::<_, i64>(0) != new.expected_lease_fence
        || lease.get::<_, String>(1) != new.source_instance
    {
        return Err(StoreError::invalid_state(
            "stale profile lease fence or holder",
        ));
    }
    let current = tx
        .query_opt(
            "select revision from player_profile_snapshots where player_uuid = $1 and scope = $2
         order by revision desc limit 1 for update",
            &[&new.player_uuid, &new.scope],
        )?
        .map(|row| row.get::<_, i64>(0))
        .unwrap_or(0);
    if current != new.expected_snapshot_revision {
        return Err(StoreError::invalid_state("stale profile snapshot revision"));
    }
    let revision = current
        .checked_add(1)
        .ok_or_else(|| StoreError::invalid_state("profile revision exhausted"))?;
    let row = tx.query_one(
        "insert into player_profile_snapshots
         (id,player_uuid,scope,revision,session_id,session_revision,lease_fence,
          expected_snapshot_revision,correlation_id,schema_name,envelope,canonical_json,
          sha256,source_instance) values
         ($1,$2,$3,$4,$5,$6,$7,$8,$9,'lkjmc-profile-one',$10,$11,$12,$13)
         returning id,player_uuid,scope,revision,session_revision,lease_fence,
          correlation_id,canonical_json,sha256",
        &[
            &new.id,
            &new.player_uuid,
            &new.scope,
            &revision,
            &new.session_id,
            &new.expected_session_revision,
            &new.expected_lease_fence,
            &new.expected_snapshot_revision,
            &new.correlation_id,
            &envelope_value,
            &canonical.json,
            &canonical.sha256,
            &new.source_instance,
        ],
    )?;
    tx.execute(
        "update player_sessions set revision = revision + 1 where id = $1",
        &[&new.session_id],
    )?;
    tx.execute(
        "update player_identities set revision = revision + 1 where player_uuid = $1",
        &[&new.player_uuid],
    )?;
    data_workflows::append(
        &mut tx,
        "profile",
        new.player_uuid,
        revision,
        new.correlation_id,
        "snapshot",
    )?;
    let result = from_row(&row, false)?;
    tx.commit()?;
    Ok(result)
}
