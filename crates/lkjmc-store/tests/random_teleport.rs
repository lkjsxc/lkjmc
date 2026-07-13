#[allow(dead_code)]
mod support;

use lkjmc_store::{migrate, player, points, random_teleport};
use random_teleport::{ReserveInput, ReserveOutcome};

#[test]
fn reserves_completes_and_refunds_idempotently() -> Result<(), Box<dyn std::error::Error>> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    let player_id = uuid::Uuid::new_v4();
    let correlation_id = uuid::Uuid::new_v4();
    let refund_id = uuid::Uuid::new_v4();
    player::insert_identity(client, player_id, "RtpPlayer")?;
    points::grant(client, player_id, 600, "test")?;

    assert_eq!(
        random_teleport::reserve(client, input(player_id, correlation_id))?,
        ReserveOutcome::Reserved
    );
    assert_eq!(points::balance(client, player_id)?, 350);
    assert_eq!(
        random_teleport::cooldown_remaining(client, player_id, "hub", "overworld", 600)?,
        600
    );
    assert!(random_teleport::complete(
        client,
        player_id,
        correlation_id
    )?);
    assert!(!random_teleport::refund(
        client,
        player_id,
        correlation_id,
        "after-success"
    )?);
    assert_eq!(
        random_teleport::reserve(client, input(player_id, refund_id))?,
        ReserveOutcome::Reserved
    );
    assert!(random_teleport::refund(
        client,
        player_id,
        refund_id,
        "test-failure"
    )?);
    assert_eq!(points::balance(client, player_id)?, 350);
    assert!(!random_teleport::refund(
        client, player_id, refund_id, "again"
    )?);
    assert_eq!(random_teleport::history(client, player_id)?.len(), 2);
    Ok(())
}

#[test]
fn duplicate_correlation_does_not_charge_twice() -> Result<(), Box<dyn std::error::Error>> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    let player_id = uuid::Uuid::new_v4();
    let correlation_id = uuid::Uuid::new_v4();
    player::insert_identity(client, player_id, "RtpPlayer")?;
    points::grant(client, player_id, 500, "test")?;
    assert_eq!(
        random_teleport::reserve(client, input(player_id, correlation_id))?,
        ReserveOutcome::Reserved
    );
    assert_eq!(
        random_teleport::reserve(client, input(player_id, correlation_id))?,
        ReserveOutcome::Existing("reserved".to_string())
    );
    assert_eq!(points::balance(client, player_id)?, 250);
    Ok(())
}

fn input(player_uuid: uuid::Uuid, correlation_id: uuid::Uuid) -> ReserveInput<'static> {
    ReserveInput {
        id: uuid::Uuid::new_v4(),
        correlation_id,
        player_uuid,
        server_id: "hub",
        profile_id: "overworld",
        world: "world",
        x: 10.0,
        y: 80.0,
        z: -20.0,
        cost_points: 250,
    }
}
