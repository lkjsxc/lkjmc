use serde_json::{json, Value};

pub(crate) struct PlanFailure {
    pub(crate) message: String,
    pub(crate) diagnostic: Value,
}

pub(crate) fn plan_failure(
    errors: Vec<String>,
    kind: &str,
    template: &str,
    attempted_queries: Vec<String>,
) -> PlanFailure {
    let code = if errors
        .iter()
        .any(|error| error.contains("invalid instance id"))
    {
        "invalid_server_id"
    } else if errors
        .iter()
        .any(|error| error.contains("unsupported instance kind"))
    {
        "unsupported_kind"
    } else if errors.iter().any(|error| error.contains("EULA")) {
        "eula_missing"
    } else if errors.iter().any(|error| error.contains("launch source")) {
        "jar_asset_missing"
    } else {
        "invalid_create_plan"
    };
    let message = if code == "jar_asset_missing" {
        format!("No compatible server jar asset is registered for project/kind '{kind}'. Run `lkjmc jar sync --project {kind}` or import a jar, then retry.")
    } else {
        errors.join("; ")
    };
    let mut diagnostic = json!({"code": code, "message": message, "issues": errors});
    if code == "jar_asset_missing" {
        diagnostic["kind"] = json!(kind);
        diagnostic["template"] = json!(template);
        diagnostic["attemptedQueries"] = json!(attempted_queries);
        diagnostic["suggestedCommand"] = json!(format!("lkjmc jar sync --project {kind}"));
    }
    PlanFailure {
        message,
        diagnostic,
    }
}

pub(crate) fn failure(code: &str, message: &str, mut diagnostic: Value) -> PlanFailure {
    diagnostic["code"] = json!(code);
    diagnostic["message"] = json!(message);
    PlanFailure {
        message: message.to_string(),
        diagnostic,
    }
}
