use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, store, with_client};

type Response = lkjmc_core::command::CommandResponse;

pub fn create(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let reporter_uuid = parse_uuid(&request, "reporterUuid")?;
        let reporter_name = body_string(&request.body, "reporterName")?;
        let target_uuid = parse_uuid(&request, "targetUuid")?;
        let target_name = body_string(&request.body, "targetName")?;
        let server_id = body_string(&request.body, "serverId")?;
        let reason = body_string(&request.body, "reason")?;
        store(lkjmc_store::player::insert_identity(
            client,
            reporter_uuid,
            &reporter_name,
        ))?;
        store(lkjmc_store::player::insert_identity(
            client,
            target_uuid,
            &target_name,
        ))?;
        let id = Uuid::new_v4();
        store(lkjmc_store::reports::create(
            client,
            id,
            reporter_uuid,
            target_uuid,
            &server_id,
            &reason,
        ))?;
        Ok(api::ok(request, json!({"id": id.to_string()})))
    })
}

pub fn list(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let limit = request
            .body
            .get("limit")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(20)
            .clamp(1, 100);
        let reports = store(lkjmc_store::reports::open(client, limit))?
            .into_iter()
            .map(|report| {
                json!({
                    "id": report.id.to_string(),
                    "reporterUuid": report.reporter_uuid.to_string(),
                    "targetUuid": report.target_uuid.to_string(),
                    "serverId": report.server_id,
                    "reason": report.reason,
                    "status": report.status
                })
            })
            .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"reports": reports})))
    })
}

pub fn resolve(state: &AppState, request: CommandEnvelope) -> Response {
    close(state, request, "resolved")
}

pub fn dismiss(state: &AppState, request: CommandEnvelope) -> Response {
    close(state, request, "dismissed")
}

fn close(state: &AppState, request: CommandEnvelope, status: &'static str) -> Response {
    with_client(state, request, |_state, request, client| {
        let report_id = parse_uuid(&request, "reportId")?;
        let closed = store(lkjmc_store::reports::close(client, report_id, status))?;
        Ok(api::ok(
            request,
            json!({"reportId": report_id.to_string(), "status": status, "closed": closed}),
        ))
    })
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}
