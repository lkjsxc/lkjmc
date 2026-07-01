#[allow(dead_code)]
mod support;

use lkjmc_store::{migrate, player, points, pool, random_teleport};
use random_teleport::{ReserveInput, ReserveOutcome};

#[test]
fn reserves_completes_and_refunds_idempotently() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let mut client = pool::connect(&url)?;
    support::reset_public_schema(&mut client)?;
    migrate::apply(&mut client)?;
    let player_id = uuid::Uuid::new_v4();
    let correlation_id = uuid::Uuid::new_v4();
    let refund_id = uuid::Uuid::new_v4();
    player::insert_identity(&mut client, player_id, "RtpPlayer")?;
    points::grant(&mut client, player_id, 600, "test")?;

    assert_eq!(
        random_teleport::reserve(&mut client, input(player_id, correlation_id))?,
        ReserveOutcome::Reserved
    );
    assert_eq!(points::balance(&mut client, player_id)?, 350);
    assert_eq!(
        random_teleport::cooldown_remaining(&mut client, player_id, "hub", 600)?,
        600
    );
    assert!(random_teleport::complete(
        &mut client,
        player_id,
        correlation_id
    )?);
    assert!(!random_teleport::refund(
        &mut client,
        player_id,
        correlation_id,
        "after-success"
    )?);
    assert_eq!(
        random_teleport::reserve(&mut client, input(player_id, refund_id))?,
        ReserveOutcome::Reserved
    );
    assert!(random_teleport::refund(
        &mut client,
        player_id,
        refund_id,
        "test-failure"
    )?);
    assert_eq!(points::balance(&mut client, player_id)?, 350);
    assert!(!random_teleport::refund(
        &mut client,
        player_id,
        refund_id,
        "again"
    )?);
    assert_eq!(random_teleport::history(&mut client, player_id)?.len(), 2);
    Ok(())
}

#[test]
fn duplicate_correlation_does_not_charge_twice() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let mut client = pool::connect(&url)?;
    support::reset_public_schema(&mut client)?;
    migrate::apply(&mut client)?;
    let player_id = uuid::Uuid::new_v4();
    let correlation_id = uuid::Uuid::new_v4();
    player::insert_identity(&mut client, player_id, "RtpPlayer")?;
    points::grant(&mut client, player_id, 500, "test")?;
    assert_eq!(
        random_teleport::reserve(&mut client, input(player_id, correlation_id))?,
        ReserveOutcome::Reserved
    );
    assert_eq!(
        random_teleport::reserve(&mut client, input(player_id, correlation_id))?,
        ReserveOutcome::Existing("reserved".to_string())
    );
    assert_eq!(points::balance(&mut client, player_id)?, 250);
    Ok(())
}

fn input(player_uuid: uuid::Uuid, correlation_id: uuid::Uuid) -> ReserveInput<'static> {
    ReserveInput {
        id: uuid::Uuid::new_v4(),
        correlation_id,
        player_uuid,
        server_id: "hub",
        world: "world",
        x: 10.0,
        y: 80.0,
        z: -20.0,
        cost_points: 250,
    }
}
