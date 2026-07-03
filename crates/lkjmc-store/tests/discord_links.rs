#[allow(dead_code)]
mod support;

use lkjmc_store::{discord_links, migrate, player, pool};

#[test]
fn manages_discord_link_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let mut client = pool::connect(&url)?;
    let _schema = support::prepare_isolated_schema(&mut client)?;
    migrate::apply(&mut client)?;
    let player_id = uuid::Uuid::new_v4();
    player::insert_identity(&mut client, player_id, "DiscordPlayer")?;
    discord_links::upsert_pending(&mut client, "123", player_id)?;
    assert_eq!(
        discord_links::get(&mut client, "123")?.map(|row| row.verification_state),
        Some("pending".to_string())
    );
    assert!(discord_links::verify(&mut client, "123")?);
    assert!(discord_links::revoke(&mut client, "123")?);
    assert!(discord_links::get(&mut client, "123")?.is_some_and(|row| row.revoked));
    Ok(())
}
