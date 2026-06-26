#[allow(dead_code)]
mod support;

use lkjmc_store::{migrate, player, points, pool, votes};
use std::env;
use support::reset_public_schema;
use uuid::Uuid;

#[test]
fn vote_reward_grants_points() -> Result<(), lkjmc_store::error::StoreError> {
    let database_url = match env::var("LKJMC_STORE_TEST_DATABASE_URL") {
        Ok(value) => value,
        Err(_) => return Ok(()),
    };
    let mut client = pool::connect(&database_url)?;
    reset_public_schema(&mut client)?;
    migrate::apply(&mut client)?;
    let player_id = Uuid::new_v4();
    player::insert_identity(&mut client, player_id, "VoteTester")?;
    votes::upsert(&mut client, "top", "vote.top", "https://vote.example", 0)?;
    votes::reward(&mut client, player_id, "VoteTester", "top", 9, "test")?;
    assert_eq!(points::balance(&mut client, player_id)?, 9);
    Ok(())
}
