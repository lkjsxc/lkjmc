use postgres::Client;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::StoreError;

use super::{catalog::item_from_row, Purchase, ShopItem};

pub fn purchase(
    client: &mut Client,
    player: Uuid,
    item_id: &str,
    correlation: Uuid,
) -> Result<Option<Purchase>, StoreError> {
    let mut tx = client.transaction()?;
    lock(&mut tx, correlation)?;
    if let Some(row) = tx.query_opt(
        "select item_id, price_points, metadata from shop_purchases where correlation_id = $1",
        &[&correlation],
    )? {
        let item = settled_item(&mut tx, row.get(0), row.get(1), row.get(2))?;
        let refunded = refunded(&mut tx, player, correlation)?;
        tx.commit()?;
        return Ok(Some(Purchase {
            item,
            duplicate: true,
            refunded,
        }));
    }
    let Some(item) = tx
        .query_opt(
            "select id, title_key, price_points, metadata from shop_items where id = $1 for share",
            &[&item_id],
        )?
        .map(item_from_row)
    else {
        tx.commit()?;
        return Ok(None);
    };
    if crate::points::spend_with_correlation(
        &mut tx,
        player,
        item.price_points,
        "shop.purchase",
        Some(correlation),
    )?
    .is_none()
    {
        tx.commit()?;
        return Err(StoreError::invalid_state("insufficient points"));
    }
    tx.execute(
        "insert into shop_purchases (id, player_uuid, item_id, price_points, correlation_id, metadata)
         values ($1, $2, $3, $4, $5, $6)",
        &[&Uuid::new_v4(), &player, &item.id, &item.price_points, &correlation, &json!({})],
    )?;
    tx.commit()?;
    Ok(Some(Purchase {
        item,
        duplicate: false,
        refunded: false,
    }))
}

pub fn reconcile_purchase(
    client: &mut Client,
    player: Uuid,
    correlation: Uuid,
) -> Result<Option<Purchase>, StoreError> {
    let Some(row) = client.query_opt(
        "select item_id, price_points, metadata from shop_purchases where player_uuid = $1 and correlation_id = $2",
        &[&player, &correlation],
    )? else { return Ok(None); };
    let item = settled_item(client, row.get(0), row.get(1), row.get(2))?;
    Ok(Some(Purchase {
        item,
        duplicate: true,
        refunded: refunded(client, player, correlation)?,
    }))
}

pub fn refund_purchase(
    client: &mut Client,
    player: Uuid,
    correlation: Uuid,
    reason: &str,
) -> Result<bool, StoreError> {
    let mut tx = client.transaction()?;
    lock(&mut tx, correlation)?;
    let row = tx.query_opt(
        "select -delta from points_ledger where player_uuid = $1 and correlation_id = $2 and reason = 'shop.purchase' and delta < 0",
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

fn settled_item(
    client: &mut impl postgres::GenericClient,
    id: String,
    price: i64,
    metadata: Value,
) -> Result<ShopItem, StoreError> {
    let title_key = client
        .query_opt("select title_key from shop_items where id = $1", &[&id])?
        .map(|row| row.get(0))
        .unwrap_or_else(|| format!("shop.item.{id}"));
    Ok(ShopItem {
        id,
        title_key,
        price_points: price,
        metadata,
    })
}
fn refunded(
    client: &mut impl postgres::GenericClient,
    player: Uuid,
    correlation: Uuid,
) -> Result<bool, StoreError> {
    Ok(client
        .query_opt(
            "select 1 from points_ledger where player_uuid = $1 and correlation_id = $2",
            &[
                &player,
                &Uuid::new_v5(&correlation, b"shop-purchase-refund"),
            ],
        )?
        .is_some())
}
fn lock(client: &mut impl postgres::GenericClient, id: Uuid) -> Result<(), StoreError> {
    client.query_one("select pg_advisory_xact_lock(hashtext($1::text))", &[&id])?;
    Ok(())
}
