#[allow(dead_code)]
mod support;

use lkjmc_store::{migrate, player, player_settings, pool};

#[test]
fn toggles_menu_and_hud_settings() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let mut client = pool::connect(&url)?;
    let _schema = support::prepare_isolated_schema(&mut client)?;
    migrate::apply(&mut client)?;
    let player_id = uuid::Uuid::new_v4();
    player::insert_identity(&mut client, player_id, "SettingsPlayer")?;
    assert!(!player_settings::toggle_menu_enabled(
        &mut client,
        player_id
    )?);
    assert_eq!(
        player_settings::menu_enabled(&mut client, player_id)?,
        Some(false)
    );
    assert!(player_settings::toggle_menu_enabled(
        &mut client,
        player_id
    )?);
    assert!(!player_settings::toggle_hud(&mut client, player_id)?);
    assert_eq!(
        player_settings::hud_enabled(&mut client, player_id)?,
        Some(false)
    );
    Ok(())
}
