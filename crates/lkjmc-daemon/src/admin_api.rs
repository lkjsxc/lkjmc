use lkjmc_core::admin::AdminRole;
use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::json;

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, store, with_connection};

pub fn handle(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let command_name = request.command.clone();
    match command_name.as_str() {
        "admin.role.list" => role_list(request),
        "admin.grant.create" => grant(state, request),
        "admin.grant.revoke" => revoke(state, request),
        "admin.principal.inspect" => inspect(state, request),
        "admin.audit.tail" => audit_tail(state, request),
        command => api::error(
            request,
            "command.unknown",
            format!("Unknown command: {command}"),
            false,
        ),
    }
}

fn role_list(request: CommandEnvelope) -> CommandResponse {
    let roles = AdminRole::all()
        .iter()
        .map(|role| json!({"id": role.id(), "permissions": role.permissions()}))
        .collect::<Vec<_>>();
    api::ok(request, json!({"roles": roles}))
}

fn grant(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    with_connection(state, request, |_state, request, client| {
        let kind = subject_string(&request, "Kind", "principalKind")?;
        let id = subject_string(&request, "Id", "principalId")?;
        let role = body_string(&request.body, "roleId")?;
        let reason = body_string(&request.body, "reason")?;
        let grant_id = store(lkjmc_store::admin::grant_role(
            client,
            &kind,
            &id,
            &role,
            &reason,
            &actor_kind(&request),
            &request.actor.name,
        ))?;
        Ok(api::ok(request, json!({"grantId": grant_id.to_string()})))
    })
}

fn revoke(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    with_connection(state, request, |_state, request, client| {
        let kind = subject_string(&request, "Kind", "principalKind")?;
        let id = subject_string(&request, "Id", "principalId")?;
        let role = body_string(&request.body, "roleId")?;
        let reason = body_string(&request.body, "reason")?;
        let revoked = store(lkjmc_store::admin::revoke_grants(
            client,
            &kind,
            &id,
            &role,
            &reason,
            &actor_kind(&request),
            &request.actor.name,
        ))?;
        Ok(api::ok(request, json!({"revoked": revoked})))
    })
}

fn inspect(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    with_connection(state, request, |_state, request, client| {
        let kind = subject_string(&request, "Kind", "principalKind")?;
        let id = subject_string(&request, "Id", "principalId")?;
        let grants = store(lkjmc_store::admin::list_grants(client, &kind, &id))?
            .into_iter()
            .map(|grant| json!({"id": grant.id.to_string(), "roleId": grant.role_id, "reason": grant.reason}))
            .collect::<Vec<_>>();
        let permissions = store(lkjmc_store::admin::effective_permissions(
            client, &kind, &id,
        ))?;
        Ok(api::ok(
            request,
            json!({"grants": grants, "permissions": permissions}),
        ))
    })
}

fn audit_tail(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    with_connection(state, request, |_state, request, client| {
        let limit = request
            .body
            .get("lines")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(50)
            .clamp(1, 200);
        let events = store(lkjmc_store::admin::tail_audit(client, limit))?
            .into_iter()
            .map(|row| {
                json!({
                    "actorKind": row.actor_kind, "actorId": row.actor_id,
                    "action": row.action, "targetKind": row.target_kind,
                    "targetId": row.target_id, "result": row.result
                })
            })
            .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"events": events})))
    })
}

fn subject_string(
    request: &CommandEnvelope,
    suffix: &str,
    fallback: &str,
) -> Result<String, String> {
    let subject = format!("subject{suffix}");
    request
        .body
        .get(&subject)
        .or_else(|| request.body.get(fallback))
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| format!("missing string field: {subject}"))
}

fn actor_kind(request: &CommandEnvelope) -> String {
    format!("{:?}", request.actor.kind).to_ascii_lowercase()
}
