use postgres::Client;
use uuid::Uuid;

use crate::error::StoreError;

pub fn claim(client: &mut Client, player_uuid: Uuid, points: i64) -> Result<bool, StoreError> {
    let inserted = client.execute(
        "insert into player_daily_claims (player_uuid, claim_date, points)
         values ($1, current_date, $2)
         on conflict (player_uuid, claim_date) do nothing",
        &[&player_uuid, &points],
    )?;
    Ok(inserted == 1)
}
