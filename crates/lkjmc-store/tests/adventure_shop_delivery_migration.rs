#[allow(dead_code)]
mod support;

use std::env;

use lkjmc_store::{migrate, player, pool, shop};
use serde_json::json;
use uuid::Uuid;

#[test]
fn migration_canonicalizes_known_row_and_preserves_purchases(
) -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut client) = database()? else {
        return Ok(());
    };
    prepare_before_42(&mut client)?;
    client.execute(
        "insert into shop_items (id, title_key, price_points, metadata) values ($1, $2, $3, $4)",
        &[
            &"adventure-end-expedition",
            &"shop.adventure",
            &250_i64,
            &json!({"delivery":{"executor":"adventure-end-expedition"}}),
        ],
    )?;
    let player_id = Uuid::new_v4();
    player::insert_identity(&mut client, player_id, "Migration")?;
    shop::record_purchase(
        &mut client,
        player_id,
        &shop::ShopItem {
            id: "adventure-end-expedition".to_string(),
            title_key: "shop.adventure".to_string(),
            price_points: 250,
            metadata: json!({"delivery":{"executor":"adventure-end-expedition"}}),
        },
    )?;
    client.batch_execute(migration_42()?)?;
    let metadata: serde_json::Value = client
        .query_one(
            "select metadata from shop_items where id = 'adventure-end-expedition'",
            &[],
        )?
        .get(0);
    assert_eq!(metadata, shop::canonical_adventure_metadata());
    assert_eq!(
        client
            .query_one("select count(*) from shop_purchases", &[])?
            .get::<_, i64>(0),
        1
    );
    assert_constraint_rejects(&mut client, "custom", shop::canonical_adventure_metadata())?;
    assert_constraint_rejects(
        &mut client,
        "custom-retired",
        json!({"delivery":{"executor":"adventure-end-expedition"}}),
    )
}

#[test]
fn migration_rejects_preexisting_custom_adventure_and_retired_executor(
) -> Result<(), lkjmc_store::error::StoreError> {
    reject_preexisting(
        "custom",
        json!({"delivery":{"executor":"adventure","adventureId":"other"}}),
    )?;
    reject_preexisting(
        "custom",
        json!({"delivery":{"executor":"adventure-end-expedition"}}),
    )
}

#[test]
fn migration_rejects_canonical_item_without_delivery_before_constraint(
) -> Result<(), lkjmc_store::error::StoreError> {
    reject_preexisting("adventure-end-expedition", json!({}))
}

fn reject_preexisting(
    id: &str,
    metadata: serde_json::Value,
) -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut client) = database()? else {
        return Ok(());
    };
    prepare_before_42(&mut client)?;
    client.execute(
        "insert into shop_items (id, title_key, price_points, metadata) values ($1, $2, $3, $4)",
        &[&id, &"shop.custom", &1_i64, &metadata],
    )?;
    let migration = migration_42()?;
    assert!(migration.contains("migration 042 cannot canonicalize shop item"));
    let error = client.batch_execute(migration).err().ok_or_else(|| {
        lkjmc_store::error::StoreError::invalid_state("migration accepted noncanonical delivery")
    })?;
    let database = error.as_db_error().ok_or_else(|| {
        lkjmc_store::error::StoreError::invalid_state("migration did not return a database error")
    })?;
    assert!(database
        .message()
        .contains("migration 042 cannot canonicalize shop item"));
    assert_ne!(
        database.constraint(),
        Some("shop_items_adventure_delivery_check"),
        "migration must report malformed pre-constraint data"
    );
    Ok(())
}

fn assert_constraint_rejects(
    client: &mut postgres::Client,
    id: &str,
    metadata: serde_json::Value,
) -> Result<(), lkjmc_store::error::StoreError> {
    let error = client.execute(
        "insert into shop_items (id, title_key, price_points, metadata) values ($1, $2, $3, $4)",
        &[&id, &"shop.custom", &1_i64, &metadata],
    ).err().ok_or_else(|| {
        lkjmc_store::error::StoreError::invalid_state("constraint accepted adventure delivery")
    })?;
    let database = error.as_db_error().ok_or_else(|| {
        lkjmc_store::error::StoreError::invalid_state("constraint did not return a database error")
    })?;
    assert_eq!(
        database.constraint(),
        Some("shop_items_adventure_delivery_check")
    );
    Ok(())
}

fn database() -> Result<Option<postgres::Client>, lkjmc_store::error::StoreError> {
    let Ok(url) = env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(None);
    };
    let mut client = pool::connect(&url)?;
    support::prepare_isolated_schema(&mut client)?;
    Ok(Some(client))
}

fn prepare_before_42(client: &mut postgres::Client) -> Result<(), lkjmc_store::error::StoreError> {
    for migration in migrate::migrations()
        .into_iter()
        .filter(|item| item.version < 42)
    {
        client.batch_execute(migration.sql)?;
    }
    Ok(())
}

fn migration_42() -> Result<&'static str, lkjmc_store::error::StoreError> {
    migrate::migrations()
        .into_iter()
        .find(|item| item.version == 42)
        .map(|item| item.sql)
        .ok_or_else(|| {
            lkjmc_store::error::StoreError::invalid_state("migration 042 is not registered")
        })
}
