use postgres::{Client, GenericClient};
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointBalance {
    pub player_uuid: Uuid,
    pub name: String,
    pub balance: i64,
}

pub fn balance(client: &mut Client, player_uuid: Uuid) -> Result<i64, StoreError> {
    let row = client.query_opt(
        "select balance from points_accounts where player_uuid = $1",
        &[&player_uuid],
    )?;
    Ok(row.map(|row| row.get(0)).unwrap_or(0))
}

pub fn grant(
    client: &mut Client,
    player_uuid: Uuid,
    amount: i64,
    reason: &str,
) -> Result<(), StoreError> {
    grant_with_correlation(client, player_uuid, amount, reason, None).map(|_| ())
}

pub fn grant_with_correlation(
    client: &mut impl GenericClient,
    player_uuid: Uuid,
    amount: i64,
    reason: &str,
    correlation_id: Option<Uuid>,
) -> Result<Uuid, StoreError> {
    ensure_account(client, player_uuid)?;
    client.execute(
        "update points_accounts set balance = balance + $2, updated_at = now()
         where player_uuid = $1",
        &[&player_uuid, &amount],
    )?;
    let metadata = serde_json::Value::Object(Default::default());
    let ledger_id = Uuid::new_v4();
    client.execute(
        "insert into points_ledger (id, player_uuid, delta, reason, correlation_id, metadata)
         values ($1, $2, $3, $4, $5, $6)",
        &[
            &ledger_id,
            &player_uuid,
            &amount,
            &reason,
            &correlation_id,
            &metadata,
        ],
    )?;
    Ok(ledger_id)
}

pub fn spend(
    client: &mut Client,
    player_uuid: Uuid,
    amount: i64,
    reason: &str,
) -> Result<bool, StoreError> {
    Ok(spend_with_correlation(client, player_uuid, amount, reason, None)?.is_some())
}

pub fn spend_with_correlation(
    client: &mut impl GenericClient,
    player_uuid: Uuid,
    amount: i64,
    reason: &str,
    correlation_id: Option<Uuid>,
) -> Result<Option<Uuid>, StoreError> {
    ensure_account(client, player_uuid)?;
    let updated = client.execute(
        "update points_accounts set balance = balance - $2, updated_at = now()
         where player_uuid = $1 and balance >= $2",
        &[&player_uuid, &amount],
    )?;
    if updated == 0 {
        return Ok(None);
    }
    let metadata = serde_json::Value::Object(Default::default());
    let delta = -amount;
    let ledger_id = Uuid::new_v4();
    client.execute(
        "insert into points_ledger (id, player_uuid, delta, reason, correlation_id, metadata)
         values ($1, $2, $3, $4, $5, $6)",
        &[
            &ledger_id,
            &player_uuid,
            &delta,
            &reason,
            &correlation_id,
            &metadata,
        ],
    )?;
    Ok(Some(ledger_id))
}

pub fn top(client: &mut Client, limit: i64) -> Result<Vec<PointBalance>, StoreError> {
    let rows = client.query(
        "select account.player_uuid, identity.current_name, account.balance
         from points_accounts account
         join player_identities identity on identity.player_uuid = account.player_uuid
         order by account.balance desc, identity.current_name asc limit $1",
        &[&limit],
    )?;
    Ok(rows
        .into_iter()
        .map(|row| PointBalance {
            player_uuid: row.get(0),
            name: row.get(1),
            balance: row.get(2),
        })
        .collect())
}

pub fn ensure_account(
    client: &mut impl GenericClient,
    player_uuid: Uuid,
) -> Result<(), StoreError> {
    client.execute(
        "insert into points_accounts (player_uuid, balance)
         values ($1, 0)
         on conflict (player_uuid) do nothing",
        &[&player_uuid],
    )?;
    Ok(())
}
