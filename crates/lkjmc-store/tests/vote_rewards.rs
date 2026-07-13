#[allow(dead_code)]
mod support;

use lkjmc_store::{migrate, player, points, votes};
use uuid::Uuid;

#[test]
fn vote_reward_grants_points() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    let player_id = Uuid::new_v4();
    player::insert_identity(client, player_id, "VoteTester")?;
    votes::upsert(client, "top", "vote.top", "https://vote.example", 0)?;
    votes::reward(client, player_id, "VoteTester", "top", 9, "test")?;
    assert_eq!(points::balance(client, player_id)?, 9);
    Ok(())
}
