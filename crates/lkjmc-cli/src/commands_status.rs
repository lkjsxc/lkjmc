use serde_json::{json, Value};

use crate::client;
use crate::error::CliError;
use crate::format;

pub fn status(socket: &str, json_output: bool) -> Result<(), CliError> {
    let response = client::call(socket, "status", json!({}))?;
    let body = format::response_body(response)?;
    if json_output {
        return format::print_json(&body);
    }
    print_human(&body);
    Ok(())
}

fn print_human(body: &Value) {
    for line in human_lines(body) {
        println!("{line}");
    }
}

fn human_lines(body: &Value) -> Vec<String> {
    let mut lines = vec![
        format!("daemon: {}", text(body, &["daemon"]).unwrap_or("unknown")),
        format!("uptimeSeconds: {}", number_text(body, &["uptimeSeconds"])),
        format!("database: {}", database_line(body)),
        format!("instances: {}", number_text(body, &["counts", "instances"])),
        format!(
            "activeSessions: {}",
            number_text(body, &["counts", "activeSessions"])
        ),
        format!("jarAssets: {}", number_text(body, &["counts", "jarAssets"])),
        format!("roots: {}", roots_line(body)),
        format!("http: {}", http_line(body)),
        format!(
            "runtime: {}",
            text(body, &["runtime", "adapter"]).unwrap_or("unknown")
        ),
        format!(
            "commandLifecycle: admissionLimit={} externalEffects={}",
            number_text(body, &["commandLifecycle", "admissionLimit"]),
            text(body, &["commandLifecycle", "externalEffects"]).unwrap_or("unknown")
        ),
        format!(
            "reconciler: {}",
            enabled_line(body, &["reconciler", "enabled"])
        ),
    ];
    if let Some(instances) = body.get("instances").and_then(Value::as_array) {
        lines.push("instanceStates:".to_string());
        if instances.is_empty() {
            lines.push("  none".to_string());
        } else {
            lines.extend(instances.iter().map(instance_line));
        }
        if bool_value(body, &["instanceSnapshot", "truncated"]) == Some(true) {
            lines.push(format!(
                "  ... additional instances omitted after {} rows",
                number_text(body, &["instanceSnapshot", "limit"])
            ));
        }
    }
    lines
}

fn instance_line(instance: &Value) -> String {
    let reason = text(instance, &["joinDisabledReason"]).unwrap_or("");
    let reason = if reason.is_empty() { "-" } else { reason };
    format!(
        "  {} kind={} desired={} observed={} processHealthy={} ready={} joinable={} reason={} observationAgeSeconds={} diagnosticsTruncated={}",
        text(instance, &["id"]).unwrap_or("unknown"),
        text(instance, &["kind"]).unwrap_or("unknown"),
        text(instance, &["desiredState"]).unwrap_or("unknown"),
        text(instance, &["observedState"]).unwrap_or("unknown"),
        bool_text(instance, &["processHealthy"]),
        bool_text(instance, &["ready"]),
        bool_text(instance, &["joinable"]),
        reason,
        number_text(instance, &["observationAgeSeconds"]),
        bool_text(instance, &["diagnosticsTruncated"]),
    )
}

fn database_line(body: &Value) -> String {
    match bool_value(body, &["database", "configured"]) {
        Some(false) => "not configured".to_string(),
        Some(true) => match bool_value(body, &["database", "connected"]) {
            Some(true) => "connected".to_string(),
            Some(false) => format!(
                "failed ({})",
                text(body, &["database", "error"]).unwrap_or("error")
            ),
            None => "configured".to_string(),
        },
        None => "unknown".to_string(),
    }
}

fn roots_line(body: &Value) -> String {
    format!(
        "config={} data={} log={} jar={}",
        text(body, &["roots", "config"]).unwrap_or("-"),
        text(body, &["roots", "data"]).unwrap_or("-"),
        text(body, &["roots", "log"]).unwrap_or("-"),
        text(body, &["roots", "jar"]).unwrap_or("-")
    )
}

fn http_line(body: &Value) -> String {
    if bool_value(body, &["http", "enabled"]) == Some(true) {
        format!(
            "enabled {}",
            text(body, &["http", "address"]).unwrap_or("-")
        )
    } else {
        "disabled".to_string()
    }
}

fn enabled_line(body: &Value, path: &[&str]) -> &'static str {
    if bool_value(body, path) == Some(true) {
        "enabled"
    } else {
        "disabled"
    }
}

fn text<'a>(body: &'a Value, path: &[&str]) -> Option<&'a str> {
    path.iter()
        .try_fold(body, |value, key| value.get(*key))?
        .as_str()
}

fn number(body: &Value, path: &[&str]) -> Option<i64> {
    path.iter()
        .try_fold(body, |value, key| value.get(*key))?
        .as_i64()
}

fn bool_value(body: &Value, path: &[&str]) -> Option<bool> {
    path.iter()
        .try_fold(body, |value, key| value.get(*key))?
        .as_bool()
}

fn number_text(body: &Value, path: &[&str]) -> String {
    number(body, path)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn bool_text(body: &Value, path: &[&str]) -> &'static str {
    match bool_value(body, path) {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_status_does_not_turn_unknown_counts_into_zero() {
        let lines = human_lines(&json!({
            "daemon": "running",
            "uptimeSeconds": 1,
            "database": {"configured": false},
            "counts": {"instances": null, "activeSessions": null, "jarAssets": null},
            "roots": {},
            "http": {"enabled": false},
            "runtime": {"adapter": "local-process"},
            "commandLifecycle": {},
            "reconciler": {"enabled": false},
            "instances": null
        }));
        assert!(lines.iter().any(|line| line == "instances: unknown"));
        assert!(lines.iter().any(|line| line == "activeSessions: unknown"));
    }

    #[test]
    fn human_status_renders_instance_truth_fields() -> Result<(), String> {
        let lines = human_lines(&json!({
            "daemon": "running",
            "uptimeSeconds": 1,
            "database": {"configured": true, "connected": true},
            "counts": {"instances": 1, "activeSessions": 0, "jarAssets": 0},
            "roots": {},
            "http": {"enabled": false},
            "runtime": {"adapter": "local-process"},
            "commandLifecycle": {},
            "reconciler": {"enabled": true},
            "instances": [{
                "id": "survival",
                "kind": "folia",
                "desiredState": "running",
                "observedState": "process-healthy",
                "processHealthy": true,
                "ready": false,
                "joinable": false,
                "joinDisabledReason": "heartbeat-stale",
                "observationAgeSeconds": 4,
                "diagnosticsTruncated": false
            }]
        }));
        let line = lines
            .iter()
            .find(|line| line.starts_with("  survival "))
            .ok_or_else(|| "survival status line missing".to_string())?;
        assert!(line.contains("processHealthy=true"));
        assert!(line.contains("ready=false"));
        assert!(line.contains("reason=heartbeat-stale"));
        assert!(line.contains("diagnosticsTruncated=false"));
        Ok(())
    }

    #[test]
    fn human_status_reports_bounded_snapshot_truncation() {
        let lines = human_lines(&json!({
            "daemon": "running",
            "uptimeSeconds": 1,
            "database": {"configured": true, "connected": true},
            "counts": {"instances": 33, "activeSessions": 0, "jarAssets": 0},
            "roots": {},
            "http": {"enabled": false},
            "runtime": {"adapter": "local-process"},
            "commandLifecycle": {},
            "reconciler": {"enabled": true},
            "instances": [],
            "instanceSnapshot": {"truncated": true, "limit": 32}
        }));
        assert!(lines
            .iter()
            .any(|line| line == "  ... additional instances omitted after 32 rows"));
    }
}
