#[allow(dead_code)]
mod support;

use lkjmc_store::{homes, migrate, player, pool, warps};
use serde_json::json;
use uuid::Uuid;

#[test]
fn lists_homes_and_warps_in_stable_order() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let mut client = pool::connect(&url)?;
    let _schema = support::prepare_isolated_schema(&mut client)?;
    migrate::apply(&mut client)?;
    let player_id = Uuid::new_v4();
    player::insert_identity(&mut client, player_id, "Traveler")?;
    homes::upsert(
        &mut client,
        Uuid::new_v4(),
        player_id,
        "zeta",
        "hub",
        json!({"x": 1}),
    )?;
    homes::upsert(
        &mut client,
        Uuid::new_v4(),
        player_id,
        "alpha",
        "hub",
        json!({"x": 2}),
    )?;
    warps::upsert(&mut client, "spawn", "hub", json!({"x": 3}))?;
    assert_eq!(homes::list(&mut client, player_id)?[0].name, "alpha");
    assert_eq!(warps::list(&mut client)?[0].name, "spawn");
    Ok(())
}
