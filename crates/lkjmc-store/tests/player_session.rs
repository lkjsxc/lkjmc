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

#[test]
fn insert_trigger_failure_rolls_back_identity() -> Result<(), Box<dyn std::error::Error>> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    fail_session_insert(client)?;
    let player_id = uuid::Uuid::new_v4();
    assert!(player_session::insert(client, uuid::Uuid::new_v4(), player_id, "fail").is_err());
    let count: i64 = client
        .query_one(
            "select count(*) from player_identities where player_uuid = $1",
            &[&player_id],
        )?
        .get(0);
    assert_eq!(count, 0);
    Ok(())
}

#[test]
fn registered_join_trigger_failure_rolls_back_all_writes() -> Result<(), Box<dyn std::error::Error>>
{
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    let player_id = uuid::Uuid::new_v4();
    player::insert_identity(client, player_id, "Before")?;
    player_session::insert(client, uuid::Uuid::new_v4(), player_id, "hub")?;
    fail_session_insert(client)?;
    assert!(
        player_session::join(client, uuid::Uuid::new_v4(), player_id, "After", "hub",).is_err()
    );
    assert_eq!(
        player::get_identity_name(client, player_id)?.as_deref(),
        Some("Before")
    );
    assert_eq!(player_session::active_count_for_server(client, "hub")?, 1);
    Ok(())
}

fn fail_session_insert(client: &mut postgres::Client) -> Result<(), postgres::Error> {
    client.batch_execute(
        "create function fail_session_insert() returns trigger language plpgsql as $$
         begin if new.current_server = 'fail' or new.current_server = 'hub' then
         raise exception 'session failpoint'; end if; return new; end $$;
         create trigger player_session_failpoint before insert on player_sessions
         for each row execute function fail_session_insert();",
    )
}
