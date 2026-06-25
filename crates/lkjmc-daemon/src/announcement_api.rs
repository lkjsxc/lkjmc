use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, store, with_client};

type Response = lkjmc_core::command::CommandResponse;

pub fn create(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let actor_name = body_string(&request.body, "actorName")?;
        let server_id = body_string(&request.body, "serverId")?;
        let message = body_string(&request.body, "message")?;
        let id = Uuid::new_v4();
        store(lkjmc_store::announcement::create(
            client,
            id,
            &actor_name,
            &server_id,
            &message,
        ))?;
        Ok(api::ok(request, json!({"id": id.to_string()})))
    })
}

pub fn recent(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let server_id = body_string(&request.body, "serverId")?;
        let limit = request
            .body
            .get("limit")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(20)
            .clamp(1, 100);
        let announcements = store(lkjmc_store::announcement::recent(client, &server_id, limit))?
            .into_iter()
            .map(|item| {
                json!({
                    "id": item.id.to_string(),
                    "actorName": item.actor_name,
                    "serverId": item.server_id,
                    "message": item.message
                })
            })
            .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"announcements": announcements})))
    })
}
