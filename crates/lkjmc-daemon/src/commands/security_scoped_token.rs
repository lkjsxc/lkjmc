use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;

const MAX_EXPIRY_SECONDS: i64 = 24 * 60 * 60;

pub fn create(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let surface = field(&request, "surface");
    let principal_kind = field(&request, "principalKind");
    let principal_id = field(&request, "principalId");
    let output_file = field(&request, "outputFile");
    let expiry = request.body.get("expiresInSeconds").and_then(serde_json::Value::as_i64).unwrap_or(0);
    let scopes = scopes(&request);
    if !["paper", "velocity", "discord", "web"].contains(&surface.as_str()) || principal_kind.is_empty() || principal_id.is_empty() || !Path::new(&output_file).is_absolute() || scopes.is_empty() || !(1..=MAX_EXPIRY_SECONDS).contains(&expiry) {
        return api::error(request, "security.credential_invalid", "surface, principal, absolute outputFile, nonempty scopes, and bounded expiry are required", false);
    }
    if state.database_url().is_none() { return api::error(request, "database.not_configured", "Database URL is not configured", false); }
    let token = super::security_token::generate_token();
    let credential_id = Uuid::new_v4();
    let mut client = match state.database_connection() { Ok(client) => client, Err(error) => return api::error(request, "database.error", error, false) };
    if let Err(error) = lkjmc_store::daemon_token::insert(&mut *client, credential_id, &lkjmc_core::security::token_hash(&token), &surface, &principal_kind, &principal_id, &scopes, expiry) {
        return api::error(request, "security.token_create_failed", error.to_string(), false);
    }
    if let Err(error) = write_secret(&output_file, &token) {
        let _ = lkjmc_store::daemon_token::revoke(&mut *client, credential_id);
        return api::error(request, "security.token_write_failed", error, false);
    }
    api::ok(request, serde_json::to_value(lkjmc_core::security::ScopedTokenCreateResult { credential_id: credential_id.to_string(), surface, principal_kind, principal_id, scopes, output_file, expires_in_seconds: expiry, fingerprint: lkjmc_core::security::token_fingerprint(&token) }).unwrap_or_else(|_| json!({})))
}

pub fn revoke(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let Some(id) = request.body.get("credentialId").and_then(serde_json::Value::as_str).map(ToString::to_string) else { return api::error(request, "security.credential_missing", "credentialId is required", false); };
    let credential_id = match Uuid::parse_str(&id) { Ok(value) => value, Err(error) => return api::error(request, "security.credential_invalid", error.to_string(), false) };
    let mut client = match state.database_connection() { Ok(client) => client, Err(error) => return api::error(request, "database.error", error, false) };
    match lkjmc_store::daemon_token::revoke(&mut *client, credential_id) { Ok(revoked) => api::ok(request, json!({"credentialId": id, "revoked": revoked > 0})), Err(error) => api::error(request, "security.token_revoke_failed", error.to_string(), false) }
}

fn field(request: &CommandEnvelope, name: &str) -> String { request.body.get(name).and_then(serde_json::Value::as_str).unwrap_or_default().to_string() }
fn scopes(request: &CommandEnvelope) -> Vec<String> { request.body.get("scopes").and_then(serde_json::Value::as_array).map(|values| values.iter().filter_map(serde_json::Value::as_str).map(ToString::to_string).collect()).unwrap_or_default() }
fn write_secret(path: &str, token: &str) -> Result<(), String> { let path = Path::new(path); let parent = path.parent().ok_or_else(|| "credential output has no parent".to_string())?; fs::create_dir_all(parent).map_err(|error| error.to_string())?; let mut file = OpenOptions::new().create_new(true).write(true).mode(0o600).open(path).map_err(|error| format!("create credential file: {error}"))?; file.write_all(token.as_bytes()).and_then(|_| file.write_all(b"\n")).and_then(|_| file.sync_all()).map_err(|error| format!("write credential file: {error}")) }
