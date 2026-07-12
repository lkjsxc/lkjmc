#[allow(dead_code)]
mod support;

use lkjmc_store::{daily, migrate};
use uuid::Uuid;

#[test]
fn reports_today_daily_claim_status() -> Result<(), Box<dyn std::error::Error>> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    let player_id = Uuid::new_v4();
    assert!(!daily::claimed_today(client, player_id)?);
    assert!(daily::claim(client, player_id, 3)?);
    assert!(daily::claimed_today(client, player_id)?);
    Ok(())
}
