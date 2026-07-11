use postgres::{Client, GenericClient};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::StoreError;

use super::{valid_minecraft_item, Purchase, ShopItem};

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
    tx.execute(
        "insert into shop_purchases
         (id, player_uuid, item_id, price_points, correlation_id, metadata)
         values ($1, $2, $3, $4, $5, $6)",
        &[
            &Uuid::new_v4(),
            &player,
            &item.id,
            &item.price_points,
            &correlation,
            &settlement(item),
        ],
    )?;
    tx.commit()?;
    Ok(Purchase {
        item: item.clone(),
        duplicate: false,
        refundable: true,
    })
}

pub fn refund_purchase(
    client: &mut Client,
    player: Uuid,
    correlation: Uuid,
    reason: &str,
) -> Result<bool, StoreError> {
    let mut tx = client.transaction()?;
    lock(&mut tx, correlation)?;
    let settled = tx.query_opt(
        "select 1 from shop_purchases where player_uuid = $1 and correlation_id = $2",
        &[&player, &correlation],
    )?;
    if settled.is_none() {
        tx.commit()?;
        return Ok(false);
    }
    let row = tx.query_opt(
        "select -delta from points_ledger where player_uuid = $1 and correlation_id = $2
         and reason = 'shop.purchase' and delta < 0",
        &[&player, &correlation],
    )?;
    let Some(row) = row else {
        tx.commit()?;
        return Ok(false);
    };
    let refund = Uuid::new_v5(&correlation, b"shop-purchase-refund");
    if tx
        .query_opt(
            "select 1 from points_ledger where correlation_id = $1",
            &[&refund],
        )?
        .is_some()
    {
        tx.commit()?;
        return Ok(false);
    }
    crate::points::grant_with_correlation(&mut tx, player, row.get(0), reason, Some(refund))?;
    tx.commit()?;
    Ok(true)
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
    client.query_one("select pg_advisory_xact_lock(hashtext($1::text))", &[&id])?;
    Ok(())
}
