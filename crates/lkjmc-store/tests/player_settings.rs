#[allow(dead_code)]
mod support;

use lkjmc_store::{migrate, player, player_settings};

#[test]
fn toggles_menu_and_hud_settings() -> Result<(), Box<dyn std::error::Error>> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    let player_id = uuid::Uuid::new_v4();
    player::insert_identity(client, player_id, "SettingsPlayer")?;
    assert!(!player_settings::toggle_menu_enabled(client, player_id)?);
    assert_eq!(
        player_settings::menu_enabled(client, player_id)?,
        Some(false)
    );
    assert!(player_settings::toggle_menu_enabled(client, player_id)?);
    assert!(!player_settings::toggle_hud(client, player_id)?);
    assert_eq!(
        player_settings::hud_enabled(client, player_id)?,
        Some(false)
    );
    Ok(())
}
