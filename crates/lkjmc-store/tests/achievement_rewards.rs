use lkjmc_store::{achievement, mail, migrate, player, pool};
use serde_json::json;
use std::env;
use uuid::Uuid;

#[test]
fn claims_mail_reward_once() -> Result<(), lkjmc_store::error::StoreError> {
    let database_url = match env::var("LKJMC_STORE_TEST_DATABASE_URL") {
        Ok(value) => value,
        Err(_) => return Ok(()),
    };
    let mut client = pool::connect(&database_url)?;
    client.batch_execute("drop schema public cascade; create schema public")?;
    migrate::apply(&mut client)?;
    let player_uuid = Uuid::new_v4();
    player::insert_identity(&mut client, player_uuid, "Rewarded")?;
    client.execute(
        "insert into achievements (id, title_key, config) values ($1, $2, $3)",
        &[
            &"mail-test",
            &"achievement.mail-test.title",
            &json!({
                "descriptionKey": "achievement.mail-test.description",
                "category": "test",
                "iconMaterial": "CHEST",
                "criteria": {"kind": "test", "threshold": 1},
                "reward": {"mail": {"body": "You earned mail!"}}
            }),
        ],
    )?;
    client.execute(
        "insert into player_achievements (player_uuid, achievement_id, progress, claimed)
         values ($1, 'mail-test', $2, true)",
        &[&player_uuid, &json!({"current": 1})],
    )?;

    let first = achievement::claim_reward(&mut client, player_uuid, "mail-test")?;
    let second = achievement::claim_reward(&mut client, player_uuid, "mail-test")?;
    let inbox = mail::inbox(&mut client, player_uuid, 10)?;

    assert!(first.mail_delivered);
    assert!(second.already_claimed);
    assert_eq!(inbox.len(), 1);
    assert_eq!(inbox[0].body, "You earned mail!");
    Ok(())
}
