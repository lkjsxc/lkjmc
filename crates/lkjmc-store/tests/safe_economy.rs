#[allow(dead_code)]
mod support;

use lkjmc_store::{exchange, migrate, player, points, shop};
use uuid::Uuid;

fn database() -> Result<Option<support::TestDatabase>, lkjmc_store::error::StoreError> {
    let Some(mut database) = support::database()? else {
        return Ok(None);
    };
    migrate::apply(database.client_mut())?;
    Ok(Some(database))
}

#[test]
fn settled_shop_replay_uses_snapshot_after_catalog_mutation(
) -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut database) = database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    let player_id = Uuid::new_v4();
    let correlation = Uuid::new_v4();
    player::insert_identity(client, player_id, "Replay")?;
    points::grant(client, player_id, 10, "test")?;
    shop::upsert_item_with_metadata(
        client,
        "safe-item",
        "shop.safe-item",
        10,
        serde_json::json!({"delivery": {"executor": "minecraft-item", "material": "STONE", "amount": 64}}),
    )?;
    let item = shop::get_item(client, "safe-item")?.ok_or_else(missing)?;
    let first = shop::purchase(client, player_id, &item, correlation)?;
    shop::upsert_item_with_metadata(
        client,
        "safe-item",
        "shop.changed",
        1,
        serde_json::json!({"delivery": {"executor": "minecraft-item", "material": "DIAMOND", "amount": 64}}),
    )?;
    let replay = shop::replay(client, player_id, correlation)?.ok_or_else(missing)?;
    assert!(!first.duplicate && first.refundable);
    assert!(replay.duplicate && !replay.refundable);
    assert_eq!(replay.item.title_key, "shop.safe-item");
    assert_eq!(replay.item.price_points, 10);
    assert_eq!(replay.item.metadata["delivery"]["material"], "STONE");
    assert_eq!(replay.item.metadata["delivery"]["amount"], 64);
    assert_eq!(points::balance(client, player_id)?, 0);
    Ok(())
}

#[test]
fn invalid_item_metadata_never_debits_points() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut database) = database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    let player_id = Uuid::new_v4();
    player::insert_identity(client, player_id, "Invalid")?;
    points::grant(client, player_id, 20, "test")?;
    for (id, material, amount) in [
        ("bad-material", "NOT_A_MATERIAL", 1),
        ("bad-amount", "STONE", 65),
    ] {
        shop::upsert_item_with_metadata(
            client,
            id,
            "shop.invalid",
            10,
            serde_json::json!({"delivery": {
                "executor": "minecraft-item", "material": material, "amount": amount
            }}),
        )?;
        let item = shop::get_item(client, id)?.ok_or_else(missing)?;
        let error = shop::purchase(client, player_id, &item, Uuid::new_v4())
            .err()
            .ok_or_else(|| {
                lkjmc_store::error::StoreError::invalid_state("invalid delivery was accepted")
            })?;
        assert!(matches!(
            error,
            lkjmc_store::error::StoreError::InvalidState(_)
        ));
    }
    assert_eq!(points::balance(client, player_id)?, 20);
    Ok(())
}

#[test]
fn exchange_reconciliation_returns_only_the_settled_player_event(
) -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut database) = database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    let player_id = Uuid::new_v4();
    let correlation = Uuid::new_v4();
    player::insert_identity(client, player_id, "Exchange")?;
    exchange::seed_default_rates(client)?;
    exchange::commit(client, player_id, "COBBLESTONE", 2, correlation)?;
    let settled = exchange::reconcile(client, player_id, correlation)?.ok_or_else(missing)?;
    assert_eq!(settled.points_delta, 2);
    assert!(settled.duplicate);
    assert!(exchange::reconcile(client, player_id, Uuid::new_v4())?.is_none());
    assert_eq!(points::balance(client, player_id)?, 2);
    Ok(())
}

fn missing() -> lkjmc_store::error::StoreError {
    lkjmc_store::error::StoreError::invalid_state("missing settlement")
}
