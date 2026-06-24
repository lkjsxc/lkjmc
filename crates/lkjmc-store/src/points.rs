use postgres::Client;
use uuid::Uuid;

use crate::error::StoreError;

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
    ensure_account(client, player_uuid)?;
    client.execute(
        "update points_accounts set balance = balance + $2, updated_at = now()
         where player_uuid = $1",
        &[&player_uuid, &amount],
    )?;
    let metadata = serde_json::Value::Object(Default::default());
    client.execute(
        "insert into points_ledger (id, player_uuid, delta, reason, metadata)
         values ($1, $2, $3, $4, $5)",
        &[&Uuid::new_v4(), &player_uuid, &amount, &reason, &metadata],
    )?;
    Ok(())
}

pub fn spend(
    client: &mut Client,
    player_uuid: Uuid,
    amount: i64,
    reason: &str,
) -> Result<bool, StoreError> {
    ensure_account(client, player_uuid)?;
    let updated = client.execute(
        "update points_accounts set balance = balance - $2, updated_at = now()
         where player_uuid = $1 and balance >= $2",
        &[&player_uuid, &amount],
    )?;
    if updated == 0 {
        return Ok(false);
    }
    let metadata = serde_json::Value::Object(Default::default());
    let delta = -amount;
    client.execute(
        "insert into points_ledger (id, player_uuid, delta, reason, metadata)
         values ($1, $2, $3, $4, $5)",
        &[&Uuid::new_v4(), &player_uuid, &delta, &reason, &metadata],
    )?;
    Ok(true)
}

pub fn ensure_account(client: &mut Client, player_uuid: Uuid) -> Result<(), StoreError> {
    client.execute(
        "insert into points_accounts (player_uuid, balance)
         values ($1, 0)
         on conflict (player_uuid) do nothing",
        &[&player_uuid],
    )?;
    Ok(())
}
