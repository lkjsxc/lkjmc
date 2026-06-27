#[allow(dead_code)]
mod support;

use lkjmc_store::{daily, migrate, pool};
use uuid::Uuid;

#[test]
fn reports_today_daily_claim_status() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let mut client = pool::connect(&url)?;
    support::reset_public_schema(&mut client)?;
    migrate::apply(&mut client)?;
    let player_id = Uuid::new_v4();
    assert!(!daily::claimed_today(&mut client, player_id)?);
    assert!(daily::claim(&mut client, player_id, 3)?);
    assert!(daily::claimed_today(&mut client, player_id)?);
    Ok(())
}
