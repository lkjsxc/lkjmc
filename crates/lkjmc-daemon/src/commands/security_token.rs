use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use base64::Engine;
use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::audit_helpers::audit;

pub fn plan(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let plan = lkjmc_core::security::rotation_plan(state.http_token_file());
    api::ok(
        request,
        serde_json::to_value(plan).unwrap_or_else(|_| json!({})),
    )
}

pub fn status(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let token = state.http_token();
    let body = lkjmc_core::security::TokenRotationStatus {
        configured: token.as_deref().is_some_and(|value| !value.is_empty()),
        token_file: state.http_token_file(),
        fingerprint: token.as_deref().map(fingerprint),
    };
    api::ok(
        request,
        serde_json::to_value(body).unwrap_or_else(|_| json!({})),
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
    let old = state.http_token();
    let token = generate_token();
    if let Err(error) = write_secret(&path, &token) {
        return api::error(request, "security.token_write_failed", error, false);
    }
    if let Err(error) = state.set_http_token(token.clone()) {
        return api::error(request, "security.token_reload_failed", error, true);
    }
    write_audit(state, &request, old.as_deref(), &token, "succeeded");
    api::ok(
        request,
        json!({
            "tokenFile": path,
            "fingerprint": fingerprint(&token),
            "oldTokenRejected": old.as_deref().map(|value| value != token).unwrap_or(true),
            "newTokenAccepted": true,
            "consumerAction": "token-file-hot-reload"
        }),
    )
}

pub fn verify(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let token = state.http_token().unwrap_or_default();
    api::ok(
        request,
        json!({
            "configured": !token.is_empty(),
            "fingerprint": (!token.is_empty()).then(|| fingerprint(&token))
        }),
    )
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
        old.map(fingerprint).unwrap_or_else(|| "none".to_string()),
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

fn generate_token() -> String {
    let mut bytes = Vec::with_capacity(32);
    bytes.extend_from_slice(Uuid::new_v4().as_bytes());
    bytes.extend_from_slice(Uuid::new_v4().as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn write_secret(path: &str, token: &str) -> Result<(), String> {
    let path = Path::new(path);
    let parent = path
        .parent()
        .ok_or_else(|| "token file has no parent".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("create token dir: {error}"))?;
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("token");
    let tmp = parent.join(format!(".{file_name}.tmp-{}", std::process::id()));
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(&tmp)
        .map_err(|error| format!("create token tmp: {error}"))?;
    file.write_all(token.as_bytes())
        .map_err(|error| format!("write token tmp: {error}"))?;
    file.write_all(b"\n")
        .map_err(|error| format!("write token newline: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("sync token tmp: {error}"))?;
    fs::rename(&tmp, path).map_err(|error| format!("replace token file: {error}"))
}

fn fingerprint(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    format!(
        "sha256:{:02x}{:02x}{:02x}{:02x}",
        digest[0], digest[1], digest[2], digest[3]
    )
}

#[cfg(test)]
#[path = "security_token_tests.rs"]
mod security_token_tests;
