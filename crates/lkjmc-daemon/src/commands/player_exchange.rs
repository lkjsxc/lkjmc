use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, store, with_connection};

type Response = lkjmc_core::command::CommandResponse;

pub fn rates(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let rates = store(lkjmc_store::exchange::list_rates(client))?
            .into_iter()
            .map(|rate| {
                json!({
                    "material": rate.material,
                    "titleKey": rate.title_key,
                    "category": rate.category,
                    "pointsPerItem": rate.points_per_item,
                    "minAmount": rate.min_amount
                })
            })
            .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"rates": rates})))
    })
}

pub fn quote(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let material = body_string(&request.body, "material")?;
        let amount = body_i64(&request, "amount")?;
        let quote = store(lkjmc_store::exchange::quote(client, &material, amount))?;
        Ok(api::ok(
            request,
            json!({
                "material": quote.material,
                "amount": quote.amount,
                "pointsDelta": quote.points_delta
            }),
        ))
    })
}

pub fn commit(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let name = body_string(&request.body, "name")?;
        let material = body_string(&request.body, "material")?;
        let amount = body_i64(&request, "amount")?;
        let correlation_id = parse_uuid(&request, "correlationId")?;
        store(lkjmc_store::player::insert_identity(
            client,
            player_uuid,
            &name,
        ))?;
        let result = store(lkjmc_store::exchange::commit(
            client,
            player_uuid,
            &material,
            amount,
            correlation_id,
        ))?;
        store(lkjmc_store::achievement::apply_event_for_player(
            client,
            player_uuid,
            Some(&name),
            "exchange-commit",
            1,
            Some(correlation_id),
        ))?;
        store(lkjmc_store::achievement::apply_event_for_player(
            client,
            player_uuid,
            Some(&name),
            "block-exchange-total",
            amount,
            Some(correlation_id),
        ))?;
        Ok(api::ok(
            request,
            json!({
                "material": result.material,
                "amount": result.amount,
                "pointsDelta": result.points_delta,
                "correlationId": result.correlation_id.to_string(),
                "duplicate": result.duplicate
            }),
        ))
    })
}

pub fn reconcile(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let correlation_id = parse_uuid(&request, "correlationId")?;
        let Some(result) = store(lkjmc_store::exchange::reconcile(client, player_uuid, correlation_id))? else {
            return Ok(api::error(request, "exchange.correlation_not_found", "exchange correlation not found", false));
        };
        Ok(api::ok(request, json!({
            "material": result.material, "amount": result.amount,
            "pointsDelta": result.points_delta, "correlationId": correlation_id.to_string(), "duplicate": true
        })))
    })
}

pub fn seed_defaults(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        store(lkjmc_store::exchange::seed_default_rates(client))?;
        store(lkjmc_store::shop::seed_default_catalog(client))?;
        store(lkjmc_store::achievement::seed_defaults(client))?;
        Ok(api::ok(request, json!({"seeded": true})))
    })
}

fn body_i64(request: &CommandEnvelope, field: &'static str) -> Result<i64, String> {
    request
        .body
        .get(field)
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| format!("missing number field: {field}"))
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}
