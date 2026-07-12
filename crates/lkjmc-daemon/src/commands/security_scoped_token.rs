use std::path::Path;

use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::audit_helpers::audit;

#[path = "security_scoped_token_io.rs"]
mod security_scoped_token_io;
#[path = "security_scoped_token_revoke.rs"]
mod security_scoped_token_revoke;

use security_scoped_token_io::{remove_secret, write_secret};
pub use security_scoped_token_revoke::revoke;

const MAX_EXPIRY_SECONDS: i64 = 24 * 60 * 60;
const ALLOWED_SCOPES: &[&str] = &["lkjmc.admin.admin", "lkjmc.admin.operator"];

pub fn create(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let surface = field(&request, "surface");
    let principal_kind = field(&request, "principalKind");
    let principal_id = field(&request, "principalId");
    let output_file = field(&request, "outputFile");
    let expiry = request
        .body
        .get("expiresInSeconds")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(0);
    let scopes = scopes(&request);
    if !["cli", "web"].contains(&surface.as_str())
        || !["operator", "service"].contains(&principal_kind.as_str())
        || principal_kind.is_empty()
        || principal_id.is_empty()
        || !Path::new(&output_file).is_absolute()
        || !allowed_scopes(&scopes)
        || !(1..=MAX_EXPIRY_SECONDS).contains(&expiry)
    {
        return api::error(request, "security.credential_invalid", "surface, principal, absolute outputFile, nonempty scopes, and bounded expiry are required", false);
    }
    if state.database_url().is_none() {
        return api::error(
            request,
            "database.not_configured",
            "Database URL is not configured",
            false,
        );
    }
    let token = super::security_token::generate_token();
    let credential_id = Uuid::new_v4();
    let mut client = match state.database_connection() {
        Ok(client) => client,
        Err(error) => return api::error(request, "database.error", error, false),
    };
    let mut transaction = match client.transaction() {
        Ok(transaction) => transaction,
        Err(error) => return api::error(request, "database.error", error.to_string(), false),
    };
    if let Err(error) = lkjmc_store::daemon_token::insert(
        &mut transaction,
        credential_id,
        &lkjmc_core::security::token_hash(&token),
        &surface,
        &principal_kind,
        &principal_id,
        &scopes,
        expiry,
    ) {
        return api::error(
            request,
            "security.token_create_failed",
            error.to_string(),
            false,
        );
    }
    if let Err(error) = write_secret(&output_file, &token) {
        if error.created_file() && remove_secret(&output_file).is_err() {
            return cleanup_failed(request);
        }
        return api::error(
            request,
            "security.token_write_failed",
            error.to_string(),
            false,
        );
    }
    if let Err(error) = audit(
        &mut transaction,
        &request,
        "security.daemon-token.create",
        "credential",
        &credential_id.to_string(),
        "succeeded",
    ) {
        if remove_secret(&output_file).is_err() {
            return cleanup_failed(request);
        }
        return api::error(request, "security.token_audit_failed", error, false);
    }
    if transaction.commit().is_err() {
        return api::error(
            request,
            "security.token_commit_unknown",
            "credential commit status is unknown; preserve the requested file and reconcile",
            false,
        );
    }
    api::ok(
        request,
        serde_json::to_value(lkjmc_core::security::ScopedTokenCreateResult {
            credential_id: credential_id.to_string(),
            expires_in_seconds: expiry,
            fingerprint: lkjmc_core::security::token_fingerprint(&token),
        })
        .unwrap_or_else(|_| json!({})),
    )
}

fn cleanup_failed(request: CommandEnvelope) -> CommandResponse {
    api::error(
        request,
        "security.token_cleanup_failed",
        "credential creation rolled back; owner cleanup is required",
        false,
    )
}

fn field(request: &CommandEnvelope, name: &str) -> String {
    request
        .body
        .get(name)
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn allowed_scopes(scopes: &[String]) -> bool {
    !scopes.is_empty()
        && scopes
            .iter()
            .all(|scope| ALLOWED_SCOPES.contains(&scope.as_str()))
}

fn scopes(request: &CommandEnvelope) -> Vec<String> {
    request
        .body
        .get("scopes")
        .and_then(serde_json::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "security_scoped_token_tests.rs"]
mod security_scoped_token_tests;
