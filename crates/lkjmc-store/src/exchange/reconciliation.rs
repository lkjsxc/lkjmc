use postgres::Client;
use uuid::Uuid;

use crate::error::StoreError;

use super::ExchangeCommit;

pub(super) fn lock_correlation(
    client: &mut impl postgres::GenericClient,
    id: Uuid,
) -> Result<(), StoreError> {
    client.query_one("select pg_advisory_xact_lock(hashtext($1::text))", &[&id])?;
    Ok(())
}

pub fn reconcile(
    client: &mut Client,
    player_uuid: Uuid,
    correlation_id: Uuid,
) -> Result<Option<ExchangeCommit>, StoreError> {
    Ok(client
        .query_opt(
            "select material, amount, points_delta from economy_exchange_events
             where player_uuid = $1 and correlation_id = $2",
            &[&player_uuid, &correlation_id],
        )?
        .map(|row| ExchangeCommit {
            material: row.get(0),
            amount: row.get(1),
            points_delta: row.get(2),
            correlation_id,
            duplicate: true,
        }))
}
