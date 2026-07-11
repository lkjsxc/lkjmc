use base64::Engine;
use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope, CommandResponse};
use lkjmc_core::id::CommandId;
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::audit_helpers::audit;

use super::security_token_io::write_secret;

pub fn plan(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    api::ok(
        request,
        serde_json::to_value(lkjmc_core::security::rotation_plan(state.http_token_file()))
            .unwrap_or_else(|_| json!({})),
    )
}

pub fn status(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let token = state.http_token();
    let scoped_token_count = state
        .database_connection()
        .ok()
        .and_then(|mut client| lkjmc_store::daemon_token::active_count(&mut client).ok())
        .unwrap_or(0);
    api::ok(
        request,
        serde_json::to_value(lkjmc_core::security::TokenRotationStatus {
            configured: token.as_deref().is_some_and(|value| !value.is_empty()),
            token_file: state.http_token_file(),
            fingerprint: token.as_deref().map(fingerprint),
            scoped_token_count,
        })
        .unwrap_or_else(|_| json!({})),
    )
}

pub fn rotate(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let Some(path) = state.http_token_file() else {
        return api::error(
            request,
            "security.token_file_missing",
            "token file is not configured",
            false,
        );
    };
    let Some(listener) = state.http_listener() else {
        return api::error(
            request,
            "security.rotation_probe_unavailable",
            "an active loopback listener is required for rotation",
            false,
        );
    };
    let Some(old) = state.http_token() else {
        return api::error(
            request,
            "security.token_missing",
            "configured token is unavailable",
            false,
        );
    };
    let token = generate_token();
    if let Err(error) = state
        .stage_http_token(token.clone(), old.clone())
        .and_then(|_| write_secret(&path, &token))
    {
        return rotation_failure(state, request, &path, &old, &token, error);
    }
    if let Err(error) = probe(&listener, &token, true) {
        return rotation_failure(state, request, &path, &old, &token, error);
    }
    if let Err(error) = state
        .retire_previous_http_token()
        .and_then(|_| probe(&listener, &old, false))
    {
        return rotation_failure(state, request, &path, &old, &token, error);
    }
    write_audit(state, &request, Some(&old), &token, "succeeded");
    api::ok(
        request,
        json!({"tokenFile":path,"fingerprint":fingerprint(&token),"oldTokenRejected":true,"newTokenAccepted":true,"consumerAction":"restart-managed-consumers-after-probe"}),
    )
}

pub fn verify(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let token = state.http_token().unwrap_or_default();
    api::ok(
        request,
        json!({"configured":!token.is_empty(),"fingerprint":(!token.is_empty()).then(|| fingerprint(&token))}),
    )
}

fn rotation_failure(
    state: &AppState,
    request: CommandEnvelope,
    path: &str,
    old: &str,
    new: &str,
    error: String,
) -> CommandResponse {
    let rollback = rollback(state, path, old, new);
    write_audit(
        state,
        &request,
        Some(old),
        new,
        if rollback.is_ok() {
            "rolled-back"
        } else {
            "rollback-failed"
        },
    );
    api::error(
        request,
        "security.rotation_probe_failed",
        format!("{error}; rollback={}", rollback.is_ok()),
        true,
    )
}

fn rollback(state: &AppState, path: &str, old: &str, new: &str) -> Result<(), String> {
    state.stage_http_token(old.to_string(), new.to_string())?;
    write_secret(path, old)?;
    state.retire_previous_http_token()
}

fn probe(listener: &str, token: &str, expected: bool) -> Result<(), String> {
    let request = CommandEnvelope {
        request_id: CommandId::internal("rotation-probe"),
        actor: Actor {
            kind: ActorKind::Cli,
            name: "rotation-probe".into(),
        },
        command: "status".into(),
        body: json!({}),
    };
    let url = format!("http://{listener}/command");
    let result = ureq::post(&url)
        .set("authorization", &format!("Bearer {token}"))
        .send_json(serde_json::to_value(request).map_err(|error| error.to_string())?);
    match result {
        Ok(response) if expected && response.status() == 200 => Ok(()),
        Err(ureq::Error::Status(403, _)) if !expected => Ok(()),
        Ok(response) => Err(format!("loopback probe returned {}", response.status())),
        Err(error) => Err(format!("loopback probe: {error}")),
    }
}

fn write_audit(
    state: &AppState,
    request: &CommandEnvelope,
    old: Option<&str>,
    new: &str,
    result: &str,
) {
    if state.database_url().is_none() {
        return;
    }
    let Ok(mut client) = state.database_connection() else {
        return;
    };
    let target = format!(
        "{}->{}",
        old.map(fingerprint).unwrap_or_else(|| "none".into()),
        fingerprint(new)
    );
    let _ = audit(
        &mut *client,
        request,
        "security.daemon-token.rotate",
        "daemon-token",
        &target,
        result,
    );
}

pub(super) fn generate_token() -> String {
    let mut bytes = Vec::with_capacity(32);
    bytes.extend_from_slice(Uuid::new_v4().as_bytes());
    bytes.extend_from_slice(Uuid::new_v4().as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

pub(super) fn fingerprint(token: &str) -> String {
    lkjmc_core::security::token_fingerprint(token)
}

#[cfg(test)]
#[path = "security_token_tests.rs"]
mod security_token_tests;
