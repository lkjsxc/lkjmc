#[allow(dead_code)]
mod support;

use lkjmc_store::{migrate, player, player_session};

#[test]
fn counts_active_sessions_and_playtime() -> Result<(), Box<dyn std::error::Error>> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    let player_id = uuid::Uuid::new_v4();
    player::insert_identity(client, player_id, "ActionBarPlayer")?;
    player_session::insert(client, uuid::Uuid::new_v4(), player_id, "hub")?;
    assert_eq!(player_session::active_count_for_server(client, "hub")?, 1);
    assert_eq!(player_session::active_count(client)?, 1);
    assert!(player_session::playtime_seconds(client, player_id)? >= 0);
    Ok(())
}
