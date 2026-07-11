use postgres::Client;
use serde_json::{json, Value};

use crate::error::StoreError;

use super::ShopItem;

const ADVENTURE_ITEM: &str = "adventure-end-expedition";

pub fn upsert_item(
    client: &mut Client,
    id: &str,
    title_key: &str,
    price_points: i64,
) -> Result<(), StoreError> {
    upsert_item_with_metadata(
        client,
        id,
        title_key,
        price_points,
        Value::Object(Default::default()),
    )
}

pub fn upsert_item_with_metadata(
    client: &mut Client,
    id: &str,
    title_key: &str,
    price_points: i64,
    metadata: Value,
) -> Result<(), StoreError> {
    validate_delivery_metadata(id, &metadata)?;
    client.execute(
        "insert into shop_items (id, title_key, price_points, metadata)
         values ($1, $2, $3, $4)
         on conflict (id) do update set title_key = excluded.title_key,
         price_points = excluded.price_points, metadata = excluded.metadata",
        &[&id, &title_key, &price_points, &metadata],
    )?;
    Ok(())
}

pub fn validate_delivery_metadata(id: &str, metadata: &Value) -> Result<(), StoreError> {
    let executor = metadata
        .pointer("/delivery/executor")
        .and_then(Value::as_str);
    if is_canonical_adventure_delivery(id, metadata) {
        return Ok(());
    }
    if id == ADVENTURE_ITEM
        || matches!(
            executor,
            Some("adventure") | Some("adventure-end-expedition")
        )
    {
        return Err(StoreError::invalid_state(
            "noncanonical adventure delivery metadata",
        ));
    }
    Ok(())
}

pub fn is_canonical_adventure_delivery(id: &str, metadata: &Value) -> bool {
    id == ADVENTURE_ITEM && metadata == &canonical_adventure_metadata()
}

pub fn canonical_adventure_metadata() -> Value {
    json!({"delivery":{"executor":"adventure","adventureId":"end-expedition"}})
}

pub fn list_items(client: &mut Client) -> Result<Vec<ShopItem>, StoreError> {
    let rows = client.query(
        "select id, title_key, price_points, metadata from shop_items order by id",
        &[],
    )?;
    Ok(rows.into_iter().map(item_from_row).collect())
}

pub fn get_item(client: &mut Client, id: &str) -> Result<Option<ShopItem>, StoreError> {
    let row = client.query_opt(
        "select id, title_key, price_points, metadata from shop_items where id = $1",
        &[&id],
    )?;
    Ok(row.map(item_from_row))
}

pub fn valid_minecraft_item(metadata: &Value) -> bool {
    if metadata
        .pointer("/delivery/executor")
        .and_then(Value::as_str)
        != Some("minecraft-item")
    {
        return false;
    }
    let Some(material) = metadata
        .pointer("/delivery/material")
        .and_then(Value::as_str)
    else {
        return false;
    };
    let Some(amount) = metadata.pointer("/delivery/amount").and_then(Value::as_i64) else {
        return false;
    };
    amount > 0
        && lkjmc_core::economy::DEFAULT_CATALOG
            .iter()
            .any(|item| item.material == material && item.amount == amount)
}

pub fn record_purchase(
    client: &mut Client,
    player: uuid::Uuid,
    item: &ShopItem,
) -> Result<(), StoreError> {
    crate::player::ensure_identity(client, player, None)?;
    client.execute(
        "insert into shop_purchases (id, player_uuid, item_id, price_points, metadata)
         values ($1, $2, $3, $4, $5)",
        &[
            &uuid::Uuid::new_v4(),
            &player,
            &item.id,
            &item.price_points,
            &super::settlement::settlement(item),
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
            json!({"category": item.category, "delivery": {
                "executor": "minecraft-item", "material": item.material, "amount": item.amount
            }}),
        )?;
    }
    upsert_item_with_metadata(
        client,
        ADVENTURE_ITEM,
        "shop.item.adventure-end-expedition",
        250,
        canonical_adventure_metadata(),
    )
}

pub(super) fn item_from_row(row: postgres::Row) -> ShopItem {
    ShopItem {
        id: row.get(0),
        title_key: row.get(1),
        price_points: row.get(2),
        metadata: row.get(3),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_noncanonical_adventure_metadata() {
        assert!(
            validate_delivery_metadata(ADVENTURE_ITEM, &canonical_adventure_metadata()).is_ok()
        );
        assert!(validate_delivery_metadata(
            "custom",
            &json!({"delivery":{"executor":"adventure","adventureId":"end-expedition"}})
        )
        .is_err());
        assert!(validate_delivery_metadata(
            ADVENTURE_ITEM,
            &json!({"delivery":{"executor":"adventure-end-expedition"}})
        )
        .is_err());
    }
}
