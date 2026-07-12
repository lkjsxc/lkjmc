use base64::Engine;
use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::audit_helpers::audit;

use super::security_token_io::write_secret;

#[path = "security_token_rollback.rs"]
mod security_token_rollback;
use security_token_rollback::rotation_failure;

pub fn plan(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    api::ok(
        request,
        serde_json::to_value(lkjmc_core::security::rotation_plan(state.http_token_file()))
            .unwrap_or_else(|_| json!({})),
    )
}

pub fn status(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let fingerprint = state.web_bootstrap_fingerprint();
    let scoped_token_count = state
        .database_connection()
        .ok()
        .and_then(|mut client| lkjmc_store::daemon_token::active_count(&mut *client).ok())
        .unwrap_or(0);
    api::ok(
        request,
        serde_json::to_value(lkjmc_core::security::TokenRotationStatus {
            configured: state.web_bootstrap_configured(),
            token_file: state.http_token_file(),
            fingerprint,
            scoped_token_count,
        })
        .unwrap_or_else(|_| json!({})),
    )
}

pub fn rotate(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    rotate_with_writer(state, request, write_secret)
}

fn rotate_with_writer<F>(
    state: &AppState,
    request: CommandEnvelope,
    mut write: F,
) -> CommandResponse
where
    F: FnMut(&str, &str) -> Result<(), String>,
{
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
    let Some(old) = state.current_web_bootstrap() else {
        return api::error(
            request,
            "security.token_missing",
            "configured token is unavailable",
            false,
        );
    };
    let token = generate_token();
    if let Err(error) = state
        .stage_web_bootstrap(token.clone(), old.clone())
        .and_then(|_| write(&path, &token))
    {
        return rotation_failure(state, request, &path, &old, &token, error, &mut write);
    }
    if let Err(error) = probe(&listener, &token, true) {
        return rotation_failure(state, request, &path, &old, &token, error, &mut write);
    }
    if let Err(error) = state
        .retire_previous_web_bootstrap()
        .and_then(|_| probe(&listener, &old, false))
    {
        return rotation_failure(state, request, &path, &old, &token, error, &mut write);
    }
    write_audit(state, &request, Some(&old), &token, "succeeded");
    api::ok(
        request,
        json!({"tokenFile":path,"fingerprint":fingerprint(&token),"oldTokenRejected":true,"newTokenAccepted":true,"consumerAction":"restart-managed-consumers-after-probe"}),
    )
}

pub fn verify(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    api::ok(
        request,
        json!({"configured":state.web_bootstrap_configured(),"fingerprint":state.web_bootstrap_fingerprint()}),
    )
}

fn probe(listener: &str, token: &str, expected: bool) -> Result<(), String> {
    let url = format!("http://{listener}/web/login");
    let result = ureq::post(&url)
        .set("content-type", "application/x-www-form-urlencoded")
        .send_string(&format!("password={}", form_value(token)));
    match result {
        Ok(response) if expected && response.status() == 200 => Ok(()),
        Err(ureq::Error::Status(403, _)) if !expected => Ok(()),
        Ok(response) => Err(format!("loopback probe returned {}", response.status())),
        Err(error) => Err(format!("loopback probe: {error}")),
    }
}

fn form_value(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                char::from(byte).to_string()
            }
            other => format!("%{other:02X}"),
        })
        .collect()
}

pub(super) fn write_audit(
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
