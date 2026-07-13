#[allow(dead_code)]
mod support;

use lkjmc_store::{discord_links, migrate, player};

#[test]
fn manages_discord_link_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    let player_id = uuid::Uuid::new_v4();
    player::insert_identity(client, player_id, "DiscordPlayer")?;
    discord_links::upsert_pending(client, "123", player_id)?;
    assert_eq!(
        discord_links::get(client, "123")?.map(|row| row.verification_state),
        Some("pending".to_string())
    );
    assert!(discord_links::verify(client, "123")?);
    assert!(discord_links::revoke(client, "123")?);
    assert!(discord_links::get(client, "123")?.is_some_and(|row| row.revoked));
    Ok(())
}
