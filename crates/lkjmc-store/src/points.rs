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

pub fn ensure_account(client: &mut Client, player_uuid: Uuid) -> Result<(), StoreError> {
    client.execute(
        "insert into points_accounts (player_uuid, balance)
         values ($1, 0)
         on conflict (player_uuid) do nothing",
        &[&player_uuid],
    )?;
    Ok(())
}
