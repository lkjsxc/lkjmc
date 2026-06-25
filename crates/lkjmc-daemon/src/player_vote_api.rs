use lkjmc_core::command::CommandEnvelope;
use serde_json::json;

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, store, with_client};

type Response = lkjmc_core::command::CommandResponse;

pub fn list(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let links = store(lkjmc_store::votes::list(client))?
            .into_iter()
            .map(|link| {
                json!({
                    "id": link.id,
                    "titleKey": link.title_key,
                    "url": link.url,
                    "sortOrder": link.sort_order
                })
            })
            .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"links": links})))
    })
}

pub fn upsert(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let id = body_string(&request.body, "id")?;
        let title_key = body_string(&request.body, "titleKey")?;
        let url = body_string(&request.body, "url")?;
        let sort_order = request
            .body
            .get("sortOrder")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);
        let sort_order = i32::try_from(sort_order).map_err(|error| error.to_string())?;
        store(lkjmc_store::votes::upsert(
            client, &id, &title_key, &url, sort_order,
        ))?;
        Ok(api::ok(request, json!({"id": id})))
    })
}
