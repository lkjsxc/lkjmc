#[allow(dead_code)]
mod support;

use lkjmc_store::{migrate, player, player_session, pool};

#[test]
fn counts_active_sessions_and_playtime() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let mut client = pool::connect(&url)?;
    let _schema = support::prepare_isolated_schema(&mut client)?;
    migrate::apply(&mut client)?;
    let player_id = uuid::Uuid::new_v4();
    player::insert_identity(&mut client, player_id, "ActionBarPlayer")?;
    player_session::insert(&mut client, uuid::Uuid::new_v4(), player_id, "hub")?;
    assert_eq!(
        player_session::active_count_for_server(&mut client, "hub")?,
        1
    );
    assert_eq!(player_session::active_count(&mut client)?, 1);
    assert!(player_session::playtime_seconds(&mut client, player_id)? >= 0);
    Ok(())
}
