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
    println!("daemon: {}", text(body, &["daemon"]).unwrap_or("unknown"));
    println!(
        "uptimeSeconds: {}",
        number(body, &["uptimeSeconds"]).unwrap_or(0)
    );
    println!("database: {}", database_line(body));
    println!(
        "instances: {}",
        number(body, &["counts", "instances"]).unwrap_or(0)
    );
    println!(
        "activeSessions: {}",
        number(body, &["counts", "activeSessions"]).unwrap_or(0)
    );
    println!(
        "jarAssets: {}",
        number(body, &["counts", "jarAssets"]).unwrap_or(0)
    );
    println!("roots: {}", roots_line(body));
    println!("http: {}", http_line(body));
    println!(
        "reconciler: {}",
        enabled_line(body, &["reconciler", "enabled"])
    );
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
