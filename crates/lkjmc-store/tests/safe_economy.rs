#[allow(dead_code)]
mod support;

use lkjmc_store::{exchange, migrate, player, points, pool, shop};
use std::env;
use uuid::Uuid;

fn database() -> Result<Option<postgres::Client>, lkjmc_store::error::StoreError> {
    let Ok(url) = env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(None);
    };
    let mut client = pool::connect(&url)?;
    let _schema = support::prepare_isolated_schema(&mut client)?;
    migrate::apply(&mut client)?;
    Ok(Some(client))
}

#[test]
fn settled_shop_replay_uses_snapshot_after_catalog_mutation(
) -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut client) = database()? else {
        return Ok(());
    };
    let player_id = Uuid::new_v4();
    let correlation = Uuid::new_v4();
    player::insert_identity(&mut client, player_id, "Replay")?;
    points::grant(&mut client, player_id, 10, "test")?;
    shop::upsert_item_with_metadata(
        &mut client,
        "safe-item",
        "shop.safe-item",
        10,
        serde_json::json!({"delivery": {"executor": "minecraft-item", "material": "STONE", "amount": 64}}),
    )?;
    let item = shop::get_item(&mut client, "safe-item")?.ok_or_else(missing)?;
    let first = shop::purchase(&mut client, player_id, &item, correlation)?;
    shop::upsert_item_with_metadata(
        &mut client,
        "safe-item",
        "shop.changed",
        1,
        serde_json::json!({"delivery": {"executor": "minecraft-item", "material": "DIAMOND", "amount": 64}}),
    )?;
    let replay = shop::replay(&mut client, player_id, correlation)?.ok_or_else(missing)?;
    assert!(!first.duplicate && first.refundable);
    assert!(replay.duplicate && !replay.refundable);
    assert_eq!(replay.item.title_key, "shop.safe-item");
    assert_eq!(replay.item.price_points, 10);
    assert_eq!(replay.item.metadata["delivery"]["material"], "STONE");
    assert_eq!(replay.item.metadata["delivery"]["amount"], 64);
    assert_eq!(points::balance(&mut client, player_id)?, 0);
    Ok(())
}

#[test]
fn invalid_item_metadata_never_debits_points() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut client) = database()? else {
        return Ok(());
    };
    let player_id = Uuid::new_v4();
    player::insert_identity(&mut client, player_id, "Invalid")?;
    points::grant(&mut client, player_id, 20, "test")?;
    for (id, material, amount) in [
        ("bad-material", "NOT_A_MATERIAL", 1),
        ("bad-amount", "STONE", 65),
    ] {
        shop::upsert_item_with_metadata(
            &mut client,
            id,
            "shop.invalid",
            10,
            serde_json::json!({"delivery": {
                "executor": "minecraft-item", "material": material, "amount": amount
            }}),
        )?;
        let item = shop::get_item(&mut client, id)?.ok_or_else(missing)?;
        let error = shop::purchase(&mut client, player_id, &item, Uuid::new_v4())
            .expect_err("invalid delivery must be rejected");
        assert!(matches!(
            error,
            lkjmc_store::error::StoreError::InvalidState(_)
        ));
    }
    assert_eq!(points::balance(&mut client, player_id)?, 20);
    Ok(())
}

#[test]
fn exchange_reconciliation_returns_only_the_settled_player_event(
) -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut client) = database()? else {
        return Ok(());
    };
    let player_id = Uuid::new_v4();
    let correlation = Uuid::new_v4();
    player::insert_identity(&mut client, player_id, "Exchange")?;
    exchange::seed_default_rates(&mut client)?;
    exchange::commit(&mut client, player_id, "COBBLESTONE", 2, correlation)?;
    let settled = exchange::reconcile(&mut client, player_id, correlation)?.ok_or_else(missing)?;
    assert_eq!(settled.points_delta, 2);
    assert!(settled.duplicate);
    assert!(exchange::reconcile(&mut client, player_id, Uuid::new_v4())?.is_none());
    assert_eq!(points::balance(&mut client, player_id)?, 2);
    Ok(())
}

fn missing() -> lkjmc_store::error::StoreError {
    lkjmc_store::error::StoreError::invalid_state("missing settlement")
}
