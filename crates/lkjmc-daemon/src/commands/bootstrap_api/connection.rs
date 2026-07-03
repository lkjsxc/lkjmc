use lkjmc_core::config::JavaEntry;
use serde_json::{json, Value};

use crate::app::AppState;

pub(super) fn body(state: &AppState) -> Result<Value, String> {
    let entry = state
        .runtime_config()?
        .map(|config| config.network.java_entry)
        .unwrap_or_default();
    let display = entry.display_socket();
    let next = next_text(&entry);
    let diagnostics = diagnostics(&entry);
    Ok(json!({
        "java": {
            "bindHost": entry.bind_host,
            "port": entry.port,
            "publicHosts": entry.public_hosts,
            "preferredPublicHost": entry.preferred_public_host,
            "display": display,
            "next": next,
            "diagnostics": diagnostics
        }
    }))
}

fn next_text(entry: &JavaEntry) -> String {
    format!("Connect to {} with a Java client.", entry.display_socket())
}

fn diagnostics(entry: &JavaEntry) -> Vec<Value> {
    if entry.preferred_host().is_some() {
        return vec![json!({"severity":"ok","message":"public host is configured"})];
    }
    vec![
        json!({"severity":"info","message":"no public host configured; using local display address"}),
    ]
}
