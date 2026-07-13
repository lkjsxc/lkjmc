use postgres::Client;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeaseRecord {
    pub player_uuid: Uuid,
    pub scope: String,
    pub holder: String,
    pub fence: i64,
    pub correlation_id: Uuid,
    pub replay: bool,
}

pub fn acquire_lease(
    client: &mut Client,
    player_uuid: Uuid,
    scope: &str,
    holder: &str,
    correlation_id: Uuid,
) -> Result<LeaseRecord, StoreError> {
    if scope.is_empty() || holder.is_empty() {
        return Err(StoreError::invalid_state(
            "lease scope and holder are required",
        ));
    }
    let mut tx = client.transaction()?;
    let row = tx.query_opt(
        "select holder, fence, correlation_id, expires_at > now()
         from player_profile_leases where player_uuid = $1 and scope = $2 for update",
        &[&player_uuid, &scope],
    )?;
    let (fence, replay) = if let Some(row) = row {
        let old_holder: String = row.get(0);
        let old_fence: i64 = row.get(1);
        let old_correlation: Uuid = row.get(2);
        let active: bool = row.get(3);
        if old_correlation == correlation_id {
            if old_holder != holder {
                return Err(StoreError::invalid_state("changed lease replay"));
            }
            (old_fence, true)
        } else {
            if active && old_holder != holder {
                return Err(StoreError::invalid_state("profile lease is held"));
            }
            let next = old_fence
                .checked_add(1)
                .ok_or_else(|| StoreError::invalid_state("lease fence exhausted"))?;
            tx.execute(
                "update player_profile_leases set holder = $3, fence = $4,
                 correlation_id = $5, expires_at = now() + interval '30 seconds', updated_at = now()
                 where player_uuid = $1 and scope = $2",
                &[&player_uuid, &scope, &holder, &next, &correlation_id],
            )?;
            (next, false)
        }
    } else {
        tx.execute(
            "insert into player_profile_leases
             (player_uuid, scope, holder, fence, expires_at, correlation_id)
             values ($1,$2,$3,1,now() + interval '30 seconds',$4)",
            &[&player_uuid, &scope, &holder, &correlation_id],
        )?;
        (1, false)
    };
    if replay {
        tx.execute(
            "update player_profile_leases set expires_at = now() + interval '30 seconds',
             updated_at = now() where player_uuid = $1 and scope = $2",
            &[&player_uuid, &scope],
        )?;
    }
    tx.commit()?;
    Ok(LeaseRecord {
        player_uuid,
        scope: scope.into(),
        holder: holder.into(),
        fence,
        correlation_id,
        replay,
    })
}
