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

use security_scoped_token_io::{ensure_private_parent, remove_secret, write_secret};
pub use security_scoped_token_revoke::revoke;

const ADMIN_MAX_EXPIRY_SECONDS: i64 = 24 * 60 * 60;
const PLUGIN_MAX_EXPIRY_SECONDS: i64 = 365 * 24 * 60 * 60;
const ADMIN_SCOPES: &[&str] = &["lkjmc.admin.admin", "lkjmc.admin.operator"];
const HEARTBEAT_SCOPE: &str = "lkjmc.instance.heartbeat";
const PLUGIN_CREDENTIAL_ROOT: &str = "/var/lib/lkjmc/private/plugin-credentials";

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
    if !valid_credential_request(
        &surface,
        &principal_kind,
        &principal_id,
        &output_file,
        &scopes,
    ) || !(1..=max_expiry_seconds(&surface)).contains(&expiry)
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
    let prepared = !matches!(surface.as_str(), "paper" | "velocity")
        || ensure_private_parent(&output_file).is_ok();
    if !prepared {
        return api::error(
            request,
            "security.token_write_failed",
            "plugin credential directory is unavailable",
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

fn max_expiry_seconds(surface: &str) -> i64 {
    if matches!(surface, "paper" | "velocity") {
        PLUGIN_MAX_EXPIRY_SECONDS
    } else {
        ADMIN_MAX_EXPIRY_SECONDS
    }
}

fn valid_credential_request(
    surface: &str,
    principal_kind: &str,
    principal_id: &str,
    output_file: &str,
    scopes: &[String],
) -> bool {
    if principal_id.is_empty() || !Path::new(output_file).is_absolute() || scopes.is_empty() {
        return false;
    }
    match surface {
        "cli" | "web" => {
            matches!(principal_kind, "operator" | "service")
                && scopes
                    .iter()
                    .all(|scope| ADMIN_SCOPES.contains(&scope.as_str()))
        }
        "paper" => {
            matches!(principal_id, "hub" | "survival")
                && valid_plugin_credential(principal_kind, principal_id, output_file, scopes)
        }
        "velocity" => {
            principal_id == "proxy"
                && valid_plugin_credential(principal_kind, principal_id, output_file, scopes)
        }
        _ => false,
    }
}

fn valid_plugin_credential(
    principal_kind: &str,
    principal_id: &str,
    output_file: &str,
    scopes: &[String],
) -> bool {
    principal_kind == "instance"
        && valid_instance_id(principal_id)
        && scopes.len() == 1
        && scopes[0] == HEARTBEAT_SCOPE
        && [
            format!("{PLUGIN_CREDENTIAL_ROOT}/{principal_id}.secret"),
            format!("{PLUGIN_CREDENTIAL_ROOT}/{principal_id}.next.secret"),
        ]
        .iter()
        .any(|expected| output_file == expected)
}

fn valid_instance_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
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
