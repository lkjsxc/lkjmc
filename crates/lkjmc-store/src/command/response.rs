use lkjmc_core::command::{CommandEnvelope, CommandErrorBody, CommandResponse};
use serde_json::{json, Value};

use crate::error::StoreError;

pub(super) fn metadata(response: &CommandResponse) -> Result<Value, StoreError> {
    serde_json::to_value(response)
        .map(|response| json!({"response": response}))
        .map_err(|error| StoreError::invalid_state(error.to_string()))
}

pub(super) fn from_metadata(metadata: Value) -> Result<CommandResponse, StoreError> {
    serde_json::from_value(metadata.get("response").cloned().unwrap_or(Value::Null))
        .map_err(|error| StoreError::invalid_state(error.to_string()))
}

pub(super) fn success(request: &CommandEnvelope, body: Value) -> CommandResponse {
    CommandResponse {
        request_id: request.request_id.clone(),
        ok: true,
        body: Some(body),
        error: None,
    }
}

pub(super) fn failure(
    request: &CommandEnvelope,
    code: &str,
    message: impl Into<String>,
    retryable: bool,
) -> CommandResponse {
    CommandResponse {
        request_id: request.request_id.clone(),
        ok: false,
        body: None,
        error: Some(CommandErrorBody {
            code: code.into(),
            message: message.into(),
            retryable,
        }),
    }
}
