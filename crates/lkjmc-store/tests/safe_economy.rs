#[allow(dead_code)]
mod support;

use lkjmc_store::{exchange, migrate, player, points, pool, shop};
use std::env;
use uuid::Uuid;

fn database() -> Result<Option<postgres::Client>, lkjmc_store::error::StoreError> {
    let Ok(url) = env::var("LKJMC_STORE_TEST_DATABASE_URL") else { return Ok(None); };
    let mut client = pool::connect(&url)?;
    let _schema = support::prepare_isolated_schema(&mut client)?;
    migrate::apply(&mut client)?;
    Ok(Some(client))
}

fn ledger(client: &mut postgres::Client, player_id: Uuid) -> Result<i64, lkjmc_store::error::StoreError> {
    Ok(client.query_one("select coalesce(sum(delta), 0) from points_ledger where player_uuid = $1", &[&player_id])?.get(0))
}

#[test]
fn balance_ledger_invariant() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut client) = database()? else { return Ok(()); };
    let player_id = Uuid::new_v4();
    player::insert_identity(&mut client, player_id, "Economist")?;
    exchange::seed_default_rates(&mut client)?;
    exchange::commit(&mut client, player_id, "COBBLESTONE", 64, Uuid::new_v4())?;
    assert_eq!(points::balance(&mut client, player_id)?, ledger(&mut client, player_id)?);
    Ok(())
}

#[test]
fn duplicate_correlation_safe() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut client) = database()? else { return Ok(()); };
    let player_id = Uuid::new_v4(); let correlation = Uuid::new_v4();
    player::insert_identity(&mut client, player_id, "Replay")?; exchange::seed_default_rates(&mut client)?;
    assert!(!exchange::commit(&mut client, player_id, "COBBLESTONE", 2, correlation)?.duplicate);
    assert!(exchange::commit(&mut client, player_id, "COBBLESTONE", 2, correlation)?.duplicate);
    assert_eq!(points::balance(&mut client, player_id)?, 2);
    Ok(())
}

#[test]
fn catalog_price_authoritative() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut client) = database()? else { return Ok(()); };
    let player_id = Uuid::new_v4();
    player::insert_identity(&mut client, player_id, "Buyer")?;
    points::grant(&mut client, player_id, 10, "test")?;
    shop::upsert_item_with_metadata(&mut client, "safe-item", "safe", 10,
        serde_json::json!({"delivery":{"executor":"minecraft-item","material":"STONE","amount":1},"reward":999}))?;
    let bought = shop::purchase(&mut client, player_id, "safe-item", Uuid::new_v4())?.unwrap();
    assert_eq!(bought.item.price_points, 10);
    assert_eq!(points::balance(&mut client, player_id)?, 0);
    Ok(())
}

#[test]
fn client_reward_ignored() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut client) = database()? else { return Ok(()); };
    let player_id = Uuid::new_v4(); player::insert_identity(&mut client, player_id, "NoReward")?;
    points::grant(&mut client, player_id, 5, "test")?;
    shop::upsert_item(&mut client, "fixed", "fixed", 5)?;
    let purchase = shop::purchase(&mut client, player_id, "fixed", Uuid::new_v4())?.unwrap();
    assert_eq!(purchase.item.price_points, 5);
    assert_eq!(points::balance(&mut client, player_id)?, 0);
    Ok(())
}

#[test]
fn economy_fault_regressions() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut client) = database()? else { return Ok(()); };
    let player_id = Uuid::new_v4(); player::insert_identity(&mut client, player_id, "NoDebt")?;
    shop::upsert_item(&mut client, "costly", "costly", 1)?;
    assert!(shop::purchase(&mut client, player_id, "costly", Uuid::new_v4()).is_err());
    assert_eq!(points::balance(&mut client, player_id)?, 0);
    assert_eq!(ledger(&mut client, player_id)?, 0);
    Ok(())
}
