use postgres::{Client, GenericClient};
use uuid::Uuid;

use crate::error::StoreError;

use super::ExchangeCommit;

pub(super) fn lock_correlation(
    client: &mut impl GenericClient,
    correlation: Uuid,
) -> Result<(), StoreError> {
    client.query_one(
        "select pg_advisory_xact_lock(hashtext($1::uuid::text))",
        &[&correlation],
    )?;
    Ok(())
}

pub fn reconcile(
    client: &mut Client,
    player: Uuid,
    correlation: Uuid,
) -> Result<Option<ExchangeCommit>, StoreError> {
    let mut tx = client.transaction()?;
    lock_correlation(&mut tx, correlation)?;
    let result = tx
        .query_opt(
            "select material, amount, points_delta from economy_exchange_events
         where player_uuid = $1 and correlation_id = $2",
            &[&player, &correlation],
        )?
        .map(|row| ExchangeCommit {
            material: row.get(0),
            amount: row.get(1),
            points_delta: row.get(2),
            correlation_id: correlation,
            duplicate: true,
        });
    tx.commit()?;
    Ok(result)
}
