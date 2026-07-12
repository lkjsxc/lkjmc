use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::audit_helpers::audit;

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
    let mut transaction = match client.transaction() {
        Ok(transaction) => transaction,
        Err(error) => return api::error(request, "database.error", error.to_string(), false),
    };
    let revoked = match lkjmc_store::daemon_token::revoke(&mut transaction, credential_id) {
        Ok(revoked) => revoked,
        Err(error) => {
            return api::error(
                request,
                "security.token_revoke_failed",
                error.to_string(),
                false,
            )
        }
    };
    if revoked > 0 {
        if let Err(error) = audit(
            &mut transaction,
            &request,
            "security.daemon-token.revoke",
            "credential",
            &id,
            "succeeded",
        ) {
            return api::error(request, "security.token_audit_failed", error, false);
        }
    }
    if let Err(error) = transaction.commit() {
        return api::error(
            request,
            "security.token_revoke_failed",
            error.to_string(),
            false,
        );
    }
    api::ok(request, json!({"credentialId": id, "revoked": revoked > 0}))
}
