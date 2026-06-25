use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, store, with_client};

type Response = lkjmc_core::command::CommandResponse;

pub fn claim(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let name = body_string(&request.body, "name")?;
        let points = request
            .body
            .get("points")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(100)
            .clamp(1, 1000);
        store(lkjmc_store::player::insert_identity(
            client,
            player_uuid,
            &name,
        ))?;
        let claimed = store(lkjmc_store::daily::claim(client, player_uuid, points))?;
        if claimed {
            store(lkjmc_store::points::grant(
                client,
                player_uuid,
                points,
                "daily",
            ))?;
        }
        let balance = store(lkjmc_store::points::balance(client, player_uuid))?;
        Ok(api::ok(
            request,
            json!({"claimed": claimed, "points": points, "balance": balance}),
        ))
    })
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}
