use postgres::Client;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::StoreError;

use super::ShopItem;

pub fn upsert_item(
    client: &mut Client,
    id: &str,
    title_key: &str,
    price: i64,
) -> Result<(), StoreError> {
    upsert_item_with_metadata(
        client,
        id,
        title_key,
        price,
        Value::Object(Default::default()),
    )
}

pub fn upsert_item_with_metadata(
    client: &mut Client,
    id: &str,
    title_key: &str,
    price: i64,
    metadata: Value,
) -> Result<(), StoreError> {
    client.execute(
        "insert into shop_items (id, title_key, price_points, metadata) values ($1, $2, $3, $4)
         on conflict (id) do update set title_key = excluded.title_key, price_points = excluded.price_points,
         metadata = excluded.metadata",
        &[&id, &title_key, &price, &metadata],
    )?;
    Ok(())
}

pub fn list_items(client: &mut Client) -> Result<Vec<ShopItem>, StoreError> {
    Ok(client
        .query(
            "select id, title_key, price_points, metadata from shop_items order by id",
            &[],
        )?
        .into_iter()
        .map(item_from_row)
        .collect())
}

pub fn get_item(client: &mut Client, id: &str) -> Result<Option<ShopItem>, StoreError> {
    Ok(client
        .query_opt(
            "select id, title_key, price_points, metadata from shop_items where id = $1",
            &[&id],
        )?
        .map(item_from_row))
}

pub fn record_purchase(
    client: &mut Client,
    player: Uuid,
    item: &ShopItem,
) -> Result<(), StoreError> {
    crate::player::ensure_identity(client, player, None)?;
    client.execute(
        "insert into shop_purchases (id, player_uuid, item_id, price_points, metadata)
         values ($1, $2, $3, $4, $5)",
        &[
            &Uuid::new_v4(),
            &player,
            &item.id,
            &item.price_points,
            &json!({}),
        ],
    )?;
    Ok(())
}

pub fn seed_default_catalog(client: &mut Client) -> Result<(), StoreError> {
    lkjmc_core::economy::validate_catalog(|material| {
        lkjmc_core::economy::DEFAULT_SELL_RATES
            .iter()
            .find(|rate| rate.0 == material)
            .map(|rate| rate.1)
    })
    .map_err(StoreError::invalid_state)?;
    for item in lkjmc_core::economy::DEFAULT_CATALOG {
        upsert_item_with_metadata(
            client,
            item.id,
            &format!("shop.item.{}", item.id),
            item.price,
            json!({
                "category": item.category, "delivery": {"executor": "minecraft-item", "material": item.material, "amount": item.amount}
            }),
        )?;
    }
    for adventure in lkjmc_core::adventure::DEFAULT_ADVENTURES {
        upsert_item_with_metadata(
            client,
            &format!("adventure-{}", adventure.id),
            &format!("shop.item.adventure-{}", adventure.id),
            adventure.price_points,
            json!({
                "category":"adventures", "delivery":{"executor":"adventure","adventureId":adventure.id}
            }),
        )?;
    }
    Ok(())
}

pub(super) fn item_from_row(row: postgres::Row) -> ShopItem {
    ShopItem {
        id: row.get(0),
        title_key: row.get(1),
        price_points: row.get(2),
        metadata: row.get(3),
    }
}
