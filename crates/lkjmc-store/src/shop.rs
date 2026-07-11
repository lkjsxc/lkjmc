use postgres::Client;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShopItem {
    pub id: String,
    pub title_key: String,
    pub price_points: i64,
    pub metadata: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Purchase {
    pub item: ShopItem,
    pub duplicate: bool,
    pub refunded: bool,
}

pub fn upsert_item(client: &mut Client, id: &str, title_key: &str, price: i64) -> Result<(), StoreError> {
    upsert_item_with_metadata(client, id, title_key, price, Value::Object(Default::default()))
}

pub fn upsert_item_with_metadata(
    client: &mut Client, id: &str, title_key: &str, price: i64, metadata: Value,
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
    Ok(client.query("select id, title_key, price_points, metadata from shop_items order by id", &[])?
        .into_iter().map(item_from_row).collect())
}

pub fn get_item(client: &mut Client, id: &str) -> Result<Option<ShopItem>, StoreError> {
    Ok(client.query_opt("select id, title_key, price_points, metadata from shop_items where id = $1", &[&id])?
        .map(item_from_row))
}

pub fn purchase(
    client: &mut Client, player_uuid: Uuid, item_id: &str, correlation_id: Uuid,
) -> Result<Option<Purchase>, StoreError> {
    let mut tx = client.transaction()?;
    lock_correlation(&mut tx, correlation_id)?;
    if let Some(row) = tx.query_opt(
        "select item_id, price_points, metadata from shop_purchases where correlation_id = $1", &[&correlation_id],
    )? {
        let item = item_for_purchase(&mut tx, row.get(0), row.get(1), row.get(2))?;
        let refunded = is_refunded(&mut tx, player_uuid, correlation_id)?;
        tx.commit()?;
        return Ok(Some(Purchase { item, duplicate: true, refunded }));
    }
    let Some(item) = tx.query_opt(
        "select id, title_key, price_points, metadata from shop_items where id = $1 for share", &[&item_id],
    )?.map(item_from_row) else { tx.commit()?; return Ok(None); };
    if crate::points::spend_with_correlation(&mut tx, player_uuid, item.price_points, "shop.purchase", Some(correlation_id))?.is_none() {
        tx.commit()?;
        return Err(StoreError::invalid_state("insufficient points"));
    }
    tx.execute(
        "insert into shop_purchases (id, player_uuid, item_id, price_points, correlation_id, metadata)
         values ($1, $2, $3, $4, $5, $6)",
        &[&Uuid::new_v4(), &player_uuid, &item.id, &item.price_points, &correlation_id, &json!({})],
    )?;
    tx.commit()?;
    Ok(Some(Purchase { item, duplicate: false, refunded: false }))
}

pub fn reconcile_purchase(client: &mut Client, player_uuid: Uuid, correlation_id: Uuid) -> Result<Option<Purchase>, StoreError> {
    let Some(row) = client.query_opt(
        "select item_id, price_points, metadata from shop_purchases where player_uuid = $1 and correlation_id = $2",
        &[&player_uuid, &correlation_id],
    )? else { return Ok(None); };
    let item = item_for_purchase(client, row.get(0), row.get(1), row.get(2))?;
    Ok(Some(Purchase { item, duplicate: true, refunded: is_refunded(client, player_uuid, correlation_id)? }))
}

pub fn seed_default_catalog(client: &mut Client) -> Result<(), StoreError> {
    lkjmc_core::economy::validate_catalog(|material| lkjmc_core::economy::DEFAULT_SELL_RATES.iter()
        .find(|rate| rate.0 == material).map(|rate| rate.1)).map_err(StoreError::invalid_state)?;
    for item in lkjmc_core::economy::DEFAULT_CATALOG {
        upsert_item_with_metadata(client, item.id, &format!("shop.item.{}", item.id), item.price, json!({
            "category": item.category, "delivery": {"executor": "minecraft-item", "material": item.material, "amount": item.amount}
        }))?;
    }
    for adventure in lkjmc_core::adventure::DEFAULT_ADVENTURES {
        upsert_item_with_metadata(client, &format!("adventure-{}", adventure.id), &format!("shop.item.adventure-{}", adventure.id), adventure.price_points, json!({
            "category":"adventures", "delivery":{"executor":"adventure","adventureId":adventure.id}
        }))?;
    }
    Ok(())
}

pub fn refund_purchase(client: &mut Client, player_uuid: Uuid, correlation_id: Uuid, reason: &str) -> Result<bool, StoreError> {
    let mut tx = client.transaction()?;
    lock_correlation(&mut tx, correlation_id)?;
    let row = tx.query_opt("select -delta from points_ledger where player_uuid = $1 and correlation_id = $2 and reason = 'shop.purchase' and delta < 0", &[&player_uuid, &correlation_id])?;
    let Some(row) = row else { tx.commit()?; return Ok(false); };
    let refund_id = Uuid::new_v5(&correlation_id, b"shop-purchase-refund");
    if tx.query_opt("select 1 from points_ledger where correlation_id = $1", &[&refund_id])?.is_some() { tx.commit()?; return Ok(false); }
    crate::points::grant_with_correlation(&mut tx, player_uuid, row.get(0), reason, Some(refund_id))?;
    tx.commit()?;
    Ok(true)
}

fn item_for_purchase(client: &mut impl postgres::GenericClient, id: String, price: i64, metadata: Value) -> Result<ShopItem, StoreError> {
    let title_key = client.query_opt("select title_key from shop_items where id = $1", &[&id])?
        .map(|row| row.get(0)).unwrap_or_else(|| format!("shop.item.{id}"));
    Ok(ShopItem { id, title_key, price_points: price, metadata })
}
fn is_refunded(client: &mut impl postgres::GenericClient, player_uuid: Uuid, correlation: Uuid) -> Result<bool, StoreError> {
    Ok(client.query_opt("select 1 from points_ledger where player_uuid = $1 and correlation_id = $2", &[&player_uuid, &Uuid::new_v5(&correlation, b"shop-purchase-refund")])?.is_some())
}
fn lock_correlation(client: &mut impl postgres::GenericClient, id: Uuid) -> Result<(), StoreError> {
    client.query_one("select pg_advisory_xact_lock(hashtext($1::text))", &[&id])?; Ok(())
}
fn item_from_row(row: postgres::Row) -> ShopItem {
    ShopItem { id: row.get(0), title_key: row.get(1), price_points: row.get(2), metadata: row.get(3) }
}
