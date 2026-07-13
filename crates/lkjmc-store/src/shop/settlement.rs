use postgres::{Client, GenericClient};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::StoreError;

use super::{valid_minecraft_item, Purchase, ShopItem};

mod refund;
pub use refund::refund_purchase;

pub fn replay(
    client: &mut Client,
    player: Uuid,
    correlation: Uuid,
) -> Result<Option<Purchase>, StoreError> {
    let row = client.query_opt(
        "select player_uuid, metadata from shop_purchases where correlation_id = $1",
        &[&correlation],
    )?;
    row.map(|row| replay_row(player, row.get(0), row.get(1)))
        .transpose()
}

pub fn purchase(
    client: &mut Client,
    player: Uuid,
    item: &ShopItem,
    correlation: Uuid,
) -> Result<Purchase, StoreError> {
    let mut tx = client.transaction()?;
    lock(&mut tx, correlation)?;
    if let Some(row) = tx.query_opt(
        "select player_uuid, metadata from shop_purchases where correlation_id = $1",
        &[&correlation],
    )? {
        let purchase = replay_row(player, row.get(0), row.get(1))?;
        tx.commit()?;
        return Ok(purchase);
    }
    if !valid_minecraft_item(&item.metadata) {
        return Err(StoreError::invalid_state("invalid minecraft item delivery"));
    }
    if crate::points::spend_with_correlation(
        &mut tx,
        player,
        item.price_points,
        "shop.purchase",
        Some(correlation),
    )?
    .is_none()
    {
        return Err(StoreError::invalid_state("insufficient points"));
    }
    let purchase_id = Uuid::new_v4();
    tx.execute(
        "insert into shop_purchases
         (id, player_uuid, item_id, price_points, correlation_id, metadata)
         values ($1, $2, $3, $4, $5, $6)",
        &[
            &purchase_id,
            &player,
            &item.id,
            &item.price_points,
            &correlation,
            &settlement(item),
        ],
    )?;
    let delivery = item
        .metadata
        .get("delivery")
        .cloned()
        .ok_or_else(|| StoreError::invalid_state("missing settled delivery"))?;
    crate::data_workflows::insert_delivery(
        &mut tx,
        &crate::data_workflows::NewDelivery {
            id: Uuid::new_v4(),
            purchase_id,
            player_uuid: player,
            delivery,
            correlation_id: correlation,
        },
    )?;
    tx.commit()?;
    Ok(Purchase {
        item: item.clone(),
        duplicate: false,
        refundable: true,
    })
}

fn replay_row(player: Uuid, owner: Uuid, metadata: Value) -> Result<Purchase, StoreError> {
    if owner != player {
        return Err(StoreError::invalid_state(
            "shop correlation belongs to another player",
        ));
    }
    Ok(Purchase {
        item: item_from_settlement(&metadata)?,
        duplicate: true,
        refundable: false,
    })
}

pub(super) fn settlement(item: &ShopItem) -> Value {
    json!({"settlement": {
        "itemId": item.id,
        "titleKey": item.title_key,
        "pricePoints": item.price_points,
        "delivery": item.metadata.get("delivery").cloned().unwrap_or(Value::Null)
    }})
}

fn item_from_settlement(metadata: &Value) -> Result<ShopItem, StoreError> {
    let facts = metadata
        .get("settlement")
        .ok_or_else(|| StoreError::invalid_state("missing shop settlement"))?;
    let string = |field| {
        facts
            .get(field)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| StoreError::invalid_state(format!("missing shop {field}")))
    };
    let price = facts
        .get("pricePoints")
        .and_then(Value::as_i64)
        .ok_or_else(|| StoreError::invalid_state("missing shop price"))?;
    Ok(ShopItem {
        id: string("itemId")?,
        title_key: string("titleKey")?,
        price_points: price,
        metadata: json!({"delivery": facts.get("delivery").cloned().unwrap_or(Value::Null)}),
    })
}

fn lock(client: &mut impl GenericClient, id: Uuid) -> Result<(), StoreError> {
    client.query_one(
        "select pg_advisory_xact_lock(hashtext($1::uuid::text))",
        &[&id],
    )?;
    Ok(())
}
