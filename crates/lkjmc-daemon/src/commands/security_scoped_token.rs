use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;

pub fn create(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let surface = request
        .body
        .get("surface")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("paper");
    let scopes = scopes(&request);
    let token = super::security_token::generate_token();
    let credential_id = Uuid::new_v4();
    let Some(_database_url) = state.database_url() else {
        return api::error(
            request,
            "database.not_configured",
            "Database URL is not configured",
            false,
        );
    };
    let mut client = match state.database_connection() {
        Ok(client) => client,
        Err(error) => return api::error(request, "database.error", error, false),
    };
    let inserted = lkjmc_store::daemon_token::insert(
        &mut *client,
        credential_id,
        &lkjmc_core::security::token_hash(&token),
        surface,
        &scopes,
    );
    if let Err(error) = inserted {
        return api::error(
            request,
            "security.token_create_failed",
            error.to_string(),
            false,
        );
    }
    let result = lkjmc_core::security::ScopedTokenCreateResult {
        credential_id: credential_id.to_string(),
        surface: surface.to_string(),
        scopes,
        token: token.clone(),
        fingerprint: super::security_token::fingerprint(&token),
    };
    api::ok(
        request,
        serde_json::to_value(result).unwrap_or_else(|_| json!({})),
    )
}

pub fn revoke(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let Some(id) = request
        .body
        .get("credentialId")
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
    else {
        return api::error(
            request,
            "security.credential_missing",
            "credentialId is required",
            false,
        );
    };
    let credential_id = match Uuid::parse_str(&id) {
        Ok(value) => value,
        Err(error) => {
            return api::error(
                request,
                "security.credential_invalid",
                error.to_string(),
                false,
            )
        }
    };
    let mut client = match state.database_connection() {
        Ok(client) => client,
        Err(error) => return api::error(request, "database.error", error, false),
    };
    match lkjmc_store::daemon_token::revoke(&mut *client, credential_id) {
        Ok(revoked) => api::ok(request, json!({"credentialId": id, "revoked": revoked > 0})),
        Err(error) => api::error(
            request,
            "security.token_revoke_failed",
            error.to_string(),
            false,
        ),
    }
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
