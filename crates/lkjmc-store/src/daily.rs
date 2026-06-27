use postgres::Client;
use uuid::Uuid;

use crate::error::StoreError;

pub fn claimed_today(client: &mut Client, player_uuid: Uuid) -> Result<bool, StoreError> {
    let row = client.query_one(
        "select exists (
             select 1 from player_daily_claims
             where player_uuid = $1 and claim_date = current_date
         )",
        &[&player_uuid],
    )?;
    Ok(row.get(0))
}

pub fn claim(client: &mut Client, player_uuid: Uuid, points: i64) -> Result<bool, StoreError> {
    let inserted = client.execute(
        "insert into player_daily_claims (player_uuid, claim_date, points)
         values ($1, current_date, $2)
         on conflict (player_uuid, claim_date) do nothing",
        &[&player_uuid, &points],
    )?;
    Ok(inserted == 1)
}
