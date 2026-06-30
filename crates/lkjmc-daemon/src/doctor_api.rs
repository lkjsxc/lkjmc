use std::path::Path;

use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::{json, Value};

use crate::api;
use crate::app::AppState;

type Check = (String, bool, String);

pub fn doctor(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let checks = doctor_checks(state);
    let failures = checks
        .iter()
        .filter(|(_, ok, _)| !ok)
        .map(|(name, _, message)| format!("{name}: {message}"))
        .collect::<Vec<_>>();
    if failures.is_empty() {
        return api::ok(request, json!({"checks": checks_json(checks)}));
    }
    api::error(request, "doctor.failed", failures.join("; "), false)
}

fn doctor_checks(state: &AppState) -> Vec<Check> {
    let mut checks = Vec::new();
    checks.push(config_check(state));
    checks.extend([
        root_check("configRoot", &state.config_root()),
        root_check("dataRoot", &state.data_root()),
        root_check("logRoot", &state.log_root()),
        root_check("jarRoot", &state.jar_root()),
        socket_parent_check(&state.socket_path()),
        http_check(state.http_listener()),
        runtime_check(state),
        database_check(state.database_url()),
    ]);
    checks
}

fn config_check(state: &AppState) -> Check {
    match state.config_path() {
        Some(path) if Path::new(&path).is_file() => ok("configFile", "loaded"),
        Some(path) => fail("configFile", format!("not readable: {path}")),
        None => ok("configFile", "default path absent or not requested"),
    }
}

fn root_check(name: &str, path: &str) -> Check {
    let value = Path::new(path);
    if path.is_empty() || !value.is_absolute() {
        return fail(name, "must be an absolute path");
    }
    if value.exists() && !value.is_dir() {
        return fail(name, "exists but is not a directory");
    }
    if value.ancestors().any(Path::exists) {
        return ok(name, "path syntax and ancestor are usable");
    }
    fail(name, "no existing ancestor")
}

fn socket_parent_check(path: &str) -> Check {
    let parent = Path::new(path).parent();
    match parent {
        Some(value) if value.is_dir() => ok("socketParent", "usable"),
        Some(value) => fail(
            "socketParent",
            format!("not a directory: {}", value.display()),
        ),
        None => fail("socketParent", "missing parent"),
    }
}

fn http_check(listener: Option<String>) -> Check {
    match listener {
        Some(value) if !value.trim().is_empty() => ok("httpListener", "configured"),
        Some(_) => fail("httpListener", "blank listener"),
        None => ok("httpListener", "disabled"),
    }
}

fn runtime_check(state: &AppState) -> Check {
    match state.runtime_adapter_name() {
        Ok("local-process") => ok("runtimeAdapter", "local-process ready"),
        Ok(value) => fail("runtimeAdapter", format!("unsupported adapter: {value}")),
        Err(error) => fail("runtimeAdapter", error),
    }
}

fn database_check(database_url: Option<String>) -> Check {
    let Some(database_url) = database_url else {
        return ok("database", "not configured");
    };
    match lkjmc_store::pool::connect(&database_url) {
        Ok(_) => ok("database", "connected"),
        Err(error) => fail("database", sanitize(&error.to_string(), &database_url)),
    }
}

fn ok(name: &str, message: &str) -> Check {
    (name.to_string(), true, message.to_string())
}

fn fail(name: &str, message: impl Into<String>) -> Check {
    (name.to_string(), false, message.into())
}

fn checks_json(checks: Vec<Check>) -> Vec<Value> {
    checks
        .into_iter()
        .map(|(name, ok, message)| json!({"name": name, "ok": ok, "message": message}))
        .collect()
}

fn sanitize(message: &str, secret: &str) -> String {
    message.replace(secret, "[redacted-database-url]")
}

#[cfg(test)]
mod tests {
    use lkjmc_core::command::{Actor, ActorKind};
    use lkjmc_core::id::CommandId;

    use super::*;

    #[test]
    fn doctor_fails_when_database_connection_fails() -> Result<(), String> {
        let root = std::env::temp_dir();
        let state = AppState::with_config_path(
            Some("postgres://lkjmc:secret@127.0.0.1:1/lkjmc".to_string()),
            root.to_string_lossy().to_string(),
            root.to_string_lossy().to_string(),
            root.to_string_lossy().to_string(),
            root.to_string_lossy().to_string(),
            None,
            None,
            None,
        );
        state.with_runtime_metadata(
            root.join("lkjmc.sock").to_string_lossy().to_string(),
            None,
            false,
        )?;
        let response = doctor(&state, request().map_err(|error| error.to_string())?);
        assert!(!response.ok);
        let error = response
            .error
            .ok_or_else(|| "doctor error missing".to_string())?;
        assert_eq!(error.code, "doctor.failed");
        assert!(!error.message.contains("secret"));
        Ok(())
    }

    fn request() -> Result<CommandEnvelope, lkjmc_core::error::IdError> {
        Ok(CommandEnvelope {
            request_id: CommandId::parse("request id", "test")?,
            actor: Actor {
                kind: ActorKind::Cli,
                name: "test".to_string(),
            },
            command: "doctor".to_string(),
            body: json!({}),
        })
    }
}
