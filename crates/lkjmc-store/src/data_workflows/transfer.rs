use postgres::Client;
use uuid::Uuid;

use crate::error::StoreError;

use super::{change_feed, record, WorkflowRecord};

pub struct NewTransfer<'a> {
    pub id: Uuid,
    pub player_uuid: Uuid,
    pub session_id: Uuid,
    pub session_revision: i64,
    pub profile_revision: i64,
    pub lease_fence: i64,
    pub scope: &'a str,
    pub target_server: &'a str,
    pub correlation_id: Uuid,
}

pub fn create_transfer(
    client: &mut Client,
    new: NewTransfer<'_>,
) -> Result<WorkflowRecord, StoreError> {
    if new.target_server.is_empty() || new.scope.is_empty() {
        return Err(StoreError::invalid_state(
            "transfer target and scope are required",
        ));
    }
    let mut tx = client.transaction()?;
    if let Some(row) = tx.query_opt(
        "select id, state, revision, fence, correlation_id, player_uuid,
         session_id, session_revision, profile_revision, lease_fence, scope, target_server
         from transfer_workflows where correlation_id = $1 for update",
        &[&new.correlation_id],
    )? {
        let same = row.get::<_, Uuid>(5) == new.player_uuid
            && row.get::<_, Uuid>(6) == new.session_id
            && row.get::<_, i64>(7) == new.session_revision
            && row.get::<_, i64>(8) == new.profile_revision
            && row.get::<_, i64>(9) == new.lease_fence
            && row.get::<_, String>(10) == new.scope
            && row.get::<_, String>(11) == new.target_server;
        if !same {
            return Err(StoreError::invalid_state("changed transfer replay"));
        }
        let result = record(&row, true)?;
        tx.commit()?;
        return Ok(result);
    }
    let session = tx
        .query_opt(
            "select revision from player_sessions where id = $1 and player_uuid = $2
         and left_at is null for update",
            &[&new.session_id, &new.player_uuid],
        )?
        .ok_or_else(|| StoreError::invalid_state("active transfer session missing"))?;
    if session.get::<_, i64>(0) != new.session_revision {
        return Err(StoreError::invalid_state("stale transfer session revision"));
    }
    let lease = tx
        .query_opt(
            "select fence from player_profile_leases where player_uuid = $1 and scope = $2
         and expires_at > now() for update",
            &[&new.player_uuid, &new.scope],
        )?
        .ok_or_else(|| StoreError::invalid_state("active transfer lease missing"))?;
    if lease.get::<_, i64>(0) != new.lease_fence {
        return Err(StoreError::invalid_state("stale transfer lease fence"));
    }
    let snapshot = tx
        .query_opt(
            "select revision from player_profile_snapshots where player_uuid = $1 and scope = $2
         order by revision desc limit 1 for update",
            &[&new.player_uuid, &new.scope],
        )?
        .ok_or_else(|| StoreError::invalid_state("transfer profile missing"))?;
    if snapshot.get::<_, i64>(0) != new.profile_revision {
        return Err(StoreError::invalid_state("stale transfer profile revision"));
    }
    let row = tx.query_one(
        "insert into transfer_workflows
         (id, player_uuid, session_id, session_revision, profile_revision,
          lease_fence, scope, target_server, state, revision, fence, correlation_id)
         values ($1,$2,$3,$4,$5,$6,$7,$8,'pending_save',1,$6,$9)
         returning id, state, revision, fence, correlation_id",
        &[
            &new.id,
            &new.player_uuid,
            &new.session_id,
            &new.session_revision,
            &new.profile_revision,
            &new.lease_fence,
            &new.scope,
            &new.target_server,
            &new.correlation_id,
        ],
    )?;
    change_feed::append(
        &mut tx,
        "transfer",
        new.id,
        1,
        new.correlation_id,
        "pending_save",
    )?;
    let result = record(&row, false)?;
    tx.commit()?;
    Ok(result)
}
