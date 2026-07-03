use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, store, with_connection};

type Response = lkjmc_core::command::CommandResponse;

pub fn send(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let sender_uuid = parse_uuid(&request, "playerUuid")?;
        let sender_name = body_string(&request.body, "senderName")?;
        let recipient_name = body_string(&request.body, "recipientName")?;
        let body = body_string(&request.body, "message")?;
        store(lkjmc_store::player::insert_identity(
            client,
            sender_uuid,
            &sender_name,
        ))?;
        let recipient = store(lkjmc_store::mail::find_recipient(client, &recipient_name))?
            .ok_or_else(|| "recipient not found".to_string())?;
        let id = Uuid::new_v4();
        store(lkjmc_store::mail::send(
            client,
            id,
            recipient,
            sender_uuid,
            &sender_name,
            &body,
        ))?;
        Ok(api::ok(
            request,
            json!({"id": id.to_string(), "recipientName": recipient_name}),
        ))
    })
}

pub fn inbox(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let limit = request
            .body
            .get("limit")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(10)
            .clamp(1, 20);
        let messages = store(lkjmc_store::mail::inbox(client, player_uuid, limit))?
            .into_iter()
            .map(|mail| json!({"id": mail.id.to_string(), "senderName": mail.sender_name, "body": mail.body, "read": mail.read}))
            .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"messages": messages})))
    })
}

pub fn read(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let message_id = parse_uuid(&request, "messageId")?;
        let Some(mail) = store(lkjmc_store::mail::read(client, player_uuid, message_id))? else {
            return Ok(api::ok(request, json!({"found": false})));
        };
        Ok(api::ok(
            request,
            json!({"found": true, "id": mail.id.to_string(), "senderName": mail.sender_name, "body": mail.body}),
        ))
    })
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}
