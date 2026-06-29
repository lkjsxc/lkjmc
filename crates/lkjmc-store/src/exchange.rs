use lkjmc_core::economy::{self, ExchangeRate as CoreRate};
use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExchangeRate {
    pub id: String,
    pub material: String,
    pub title_key: String,
    pub category: String,
    pub points_per_item: i64,
    pub min_amount: i64,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExchangeCommit {
    pub material: String,
    pub amount: i64,
    pub points_delta: i64,
    pub correlation_id: Uuid,
    pub duplicate: bool,
}

pub fn list_rates(client: &mut Client) -> Result<Vec<ExchangeRate>, StoreError> {
    let rows = client.query(
        "select id, material, title_key, category, points_per_item, min_amount, enabled
         from economy_exchange_rates where enabled = true order by material",
        &[],
    )?;
    Ok(rows.into_iter().map(rate_from_row).collect())
}

pub fn upsert_rate(
    client: &mut Client,
    material: &str,
    points_per_item: i64,
    category: &str,
) -> Result<(), StoreError> {
    let material = economy::normalize_material(material).map_err(StoreError::invalid_state)?;
    client.execute(
        "insert into economy_exchange_rates
         (id, material, title_key, category, points_per_item, min_amount, enabled)
         values ($1, $2, $3, $4, $5, 1, true)
         on conflict (material) do update set
         points_per_item = excluded.points_per_item,
         category = excluded.category,
         enabled = true,
         updated_at = now()",
        &[
            &format!("material-{}", material.to_ascii_lowercase()),
            &material,
            &format!("exchange.material.{}", material.to_ascii_lowercase()),
            &category,
            &points_per_item,
        ],
    )?;
    Ok(())
}

pub fn seed_default_rates(client: &mut Client) -> Result<(), StoreError> {
    for (material, points, category) in economy::DEFAULT_SELL_RATES {
        upsert_rate(client, material, *points, category)?;
    }
    Ok(())
}

pub fn quote(
    client: &mut Client,
    material: &str,
    amount: i64,
) -> Result<ExchangeCommit, StoreError> {
    let rate = rate_by_material(client, material)?;
    let quote = economy::quote(&core_rate(&rate), amount).map_err(StoreError::invalid_state)?;
    Ok(ExchangeCommit {
        material: quote.material,
        amount: quote.amount,
        points_delta: quote.points,
        correlation_id: Uuid::nil(),
        duplicate: false,
    })
}

pub fn commit(
    client: &mut Client,
    player_uuid: Uuid,
    material: &str,
    amount: i64,
    correlation_id: Uuid,
) -> Result<ExchangeCommit, StoreError> {
    let material = economy::normalize_material(material).map_err(StoreError::invalid_state)?;
    let mut tx = client.transaction()?;
    if let Some(existing) = tx.query_opt(
        "select material, amount, points_delta from economy_exchange_events where correlation_id = $1",
        &[&correlation_id],
    )? {
        tx.commit()?;
        return Ok(ExchangeCommit {
            material: existing.get(0),
            amount: existing.get(1),
            points_delta: existing.get(2),
            correlation_id,
            duplicate: true,
        });
    }
    let rate = tx.query_one(
        "select id, material, title_key, category, points_per_item, min_amount, enabled
         from economy_exchange_rates where material = $1 and enabled = true for update",
        &[&material],
    )?;
    let rate = rate_from_row(rate);
    let quoted = economy::quote(&core_rate(&rate), amount).map_err(StoreError::invalid_state)?;
    let ledger_id = crate::points::grant_with_correlation(
        &mut tx,
        player_uuid,
        quoted.points,
        "player.exchange",
        Some(correlation_id),
    )?;
    let event_id = Uuid::new_v4();
    let metadata = Value::Object(Default::default());
    tx.execute(
        "insert into economy_exchange_events
         (id, player_uuid, rate_id, material, amount, points_delta, ledger_id, correlation_id, metadata)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        &[
            &event_id,
            &player_uuid,
            &rate.id,
            &quoted.material,
            &quoted.amount,
            &quoted.points,
            &ledger_id,
            &correlation_id,
            &metadata,
        ],
    )?;
    tx.commit()?;
    Ok(ExchangeCommit {
        material: quoted.material,
        amount: quoted.amount,
        points_delta: quoted.points,
        correlation_id,
        duplicate: false,
    })
}

fn rate_by_material(client: &mut Client, material: &str) -> Result<ExchangeRate, StoreError> {
    let material = economy::normalize_material(material).map_err(StoreError::invalid_state)?;
    client
        .query_opt(
            "select id, material, title_key, category, points_per_item, min_amount, enabled
             from economy_exchange_rates where material = $1 and enabled = true",
            &[&material],
        )?
        .map(rate_from_row)
        .ok_or_else(|| StoreError::invalid_state("exchange rate not found"))
}

fn core_rate(rate: &ExchangeRate) -> CoreRate {
    CoreRate {
        material: rate.material.clone(),
        points_per_item: rate.points_per_item,
        min_amount: rate.min_amount,
        enabled: rate.enabled,
    }
}

fn rate_from_row(row: postgres::Row) -> ExchangeRate {
    ExchangeRate {
        id: row.get(0),
        material: row.get(1),
        title_key: row.get(2),
        category: row.get(3),
        points_per_item: row.get(4),
        min_amount: row.get(5),
        enabled: row.get(6),
    }
}
