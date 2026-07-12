#[allow(dead_code)]
mod support;

use lkjmc_store::{homes, migrate, player, warps};
use serde_json::json;
use uuid::Uuid;

#[test]
fn lists_homes_and_warps_in_stable_order() -> Result<(), Box<dyn std::error::Error>> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    let player_id = Uuid::new_v4();
    player::insert_identity(client, player_id, "Traveler")?;
    homes::upsert(
        client,
        Uuid::new_v4(),
        player_id,
        "zeta",
        "hub",
        json!({"x": 1}),
    )?;
    homes::upsert(
        client,
        Uuid::new_v4(),
        player_id,
        "alpha",
        "hub",
        json!({"x": 2}),
    )?;
    warps::upsert(client, "spawn", "hub", json!({"x": 3}))?;
    assert_eq!(homes::list(client, player_id)?[0].name, "alpha");
    assert_eq!(warps::list(client)?[0].name, "spawn");
    Ok(())
}
