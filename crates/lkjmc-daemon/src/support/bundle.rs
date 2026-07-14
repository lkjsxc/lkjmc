use std::fs;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::{Duration, Instant};

use chrono::{SecondsFormat, Utc};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::app::AppState;

mod archive;

const BYTE_CAP: usize = 2 * 1024 * 1024;
const EVENT_CAP: i64 = 200;
const TIME_CAP: Duration = Duration::from_secs(7);

pub(crate) fn create(state: &AppState, output: &Path) -> Result<Value, String> {
    let started = Instant::now();
    archive::validate_output(output)?;
    let mut entries = collect(state)?;
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    let bytes = entries.iter().map(|(_, value)| value.len()).sum::<usize>();
    if bytes > BYTE_CAP {
        return Err("support bundle byte cap exceeded".into());
    }
    check_time(started)?;
    let created_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let members = entries
        .iter()
        .map(|(name, value)| {
            json!({
                "name": name, "bytes": value.len(), "sha256": hash(value)
            })
        })
        .collect::<Vec<_>>();
    let manifest = crate::support::redaction::json_bytes(&json!({
        "schemaVersion": 1, "createdAt": created_at.clone(), "source": "daemon-local",
        "eventCap": EVENT_CAP, "byteCap": BYTE_CAP, "truncated": false, "members": members
    }))?;
    if bytes.saturating_add(manifest.len()) > BYTE_CAP {
        return Err("support bundle byte cap exceeded".into());
    }
    entries.push(("manifest.json".into(), manifest));
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    archive::write(output, &entries, started, TIME_CAP)?;
    Ok(
        json!({"schemaVersion":1,"createdAt":created_at,"source":"daemon-local",
        "members":members,"eventCap":EVENT_CAP,"byteCap":BYTE_CAP,"truncated":false}),
    )
}

fn collect(state: &AppState) -> Result<Vec<(String, Vec<u8>)>, String> {
    let status =
        crate::commands::status_api::status_body(state).map_err(|_| "status unavailable")?;
    let readiness = crate::observability::health::readiness_body(state)
        .map_err(|code| format!("readiness unavailable: {code}"))?;
    let (_, in_flight) = state.admission_diagnostics();
    let metrics = state.metrics().export(in_flight).into_bytes();
    let mut client = state
        .request_database_connection()
        .map_err(|_| "events unavailable")?;
    let events = lkjmc_store::observability::query(
        &mut *client,
        lkjmc_store::observability::EventQuery {
            request_id: None,
            operation_id: None,
            correlation_id: None,
            limit: EVENT_CAP,
        },
    )
    .map_err(|_| "events unavailable")?;
    let mut output = vec![
        (
            "events.json".into(),
            crate::support::redaction::json_bytes(&json!({"events":events}))?,
        ),
        ("metrics.prom".into(), metrics),
        (
            "readiness.json".into(),
            crate::support::redaction::json_bytes(&readiness)?,
        ),
        (
            "status.json".into(),
            crate::support::redaction::json_bytes(&status)?,
        ),
    ];
    if let Some(log) = allowlisted_log(state)? {
        output.push(("files/daemon.log".into(), log));
    }
    Ok(output)
}

fn allowlisted_log(state: &AppState) -> Result<Option<Vec<u8>>, String> {
    let path = Path::new(&state.log_root()).join("daemon.log");
    let metadata = match fs::symlink_metadata(&path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("inspect allowlisted log failed".into()),
    };
    if !metadata.file_type().is_file() || metadata.len() > BYTE_CAP as u64 {
        return Err("allowlisted log is not a bounded regular file".into());
    }
    let file = fs::File::open(path).map_err(|_| "open allowlisted log failed")?;
    let opened = file
        .metadata()
        .map_err(|_| "inspect opened allowlisted log failed")?;
    if !opened.is_file() || opened.dev() != metadata.dev() || opened.ino() != metadata.ino() {
        return Err("allowlisted log changed while opening".into());
    }
    let mut value = Vec::new();
    file.take((BYTE_CAP + 1) as u64)
        .read_to_end(&mut value)
        .map_err(|_| "read allowlisted log failed")?;
    if value.len() > BYTE_CAP {
        return Err("allowlisted log grew beyond byte cap".into());
    }
    Ok(Some(crate::support::redaction::text_bytes(&value)))
}

fn check_time(started: Instant) -> Result<(), String> {
    if started.elapsed() > TIME_CAP {
        Err("support bundle time cap exceeded".into())
    } else {
        Ok(())
    }
}
fn hash(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}
