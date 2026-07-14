use std::path::Path;
use std::time::{Duration, Instant};

use chrono::{SecondsFormat, Utc};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::app::AppState;

mod archive;
mod log;

const BYTE_CAP: usize = 2 * 1024 * 1024;
const EVENT_CAP: i64 = 200;
const TIME_CAP: Duration = Duration::from_secs(7);

#[derive(Clone, Copy)]
pub(super) struct Deadline {
    expires: Instant,
}

impl Deadline {
    fn new(cap: Duration) -> Result<Self, String> {
        Instant::now()
            .checked_add(cap)
            .map(|expires| Self { expires })
            .ok_or_else(|| "support bundle deadline unavailable".into())
    }
    pub(super) fn remaining(self) -> Result<Duration, String> {
        self.expires
            .checked_duration_since(Instant::now())
            .filter(|value| !value.is_zero())
            .ok_or_else(|| "support bundle time cap exceeded".into())
    }
    pub(super) fn check(self) -> Result<(), String> {
        self.remaining().map(|_| ())
    }
}

pub(crate) fn create(state: &AppState, output: &Path) -> Result<Value, String> {
    create_inner(state, output, TIME_CAP, Duration::ZERO)
}

#[cfg(test)]
pub(crate) fn create_with_fault(
    state: &AppState,
    output: &Path,
    cap: Duration,
    archive_delay: Duration,
) -> Result<Value, String> {
    create_inner(state, output, cap, archive_delay)
}

fn create_inner(
    state: &AppState,
    output: &Path,
    cap: Duration,
    archive_delay: Duration,
) -> Result<Value, String> {
    let deadline = Deadline::new(cap)?;
    deadline.check()?;
    let output = archive::VerifiedOutput::new(output)?;
    deadline.check()?;
    let mut entries = collect(state, deadline)?;
    deadline.check()?;
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    let bytes = entries.iter().map(|(_, value)| value.len()).sum::<usize>();
    if bytes > BYTE_CAP {
        return Err("support bundle byte cap exceeded".into());
    }
    deadline.check()?;
    let created_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let mut members = Vec::with_capacity(entries.len());
    for (name, value) in &entries {
        deadline.check()?;
        members.push(json!({"name":name,"bytes":value.len(),"sha256":hash(value)}));
    }
    deadline.check()?;
    let manifest = crate::support::redaction::json_bytes(&json!({
        "schemaVersion":1,"createdAt":created_at.clone(),"source":"daemon-local",
        "eventCap":EVENT_CAP,"byteCap":BYTE_CAP,"truncated":false,"members":members
    }))?;
    if bytes.saturating_add(manifest.len()) > BYTE_CAP {
        return Err("support bundle byte cap exceeded".into());
    }
    entries.push(("manifest.json".into(), manifest));
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    deadline.check()?;
    archive::write(&output, &entries, deadline, archive_delay)?;
    Ok(
        json!({"schemaVersion":1,"createdAt":created_at,"source":"daemon-local",
        "members":members,"eventCap":EVENT_CAP,"byteCap":BYTE_CAP,"truncated":false}),
    )
}

fn collect(state: &AppState, deadline: Deadline) -> Result<Vec<(String, Vec<u8>)>, String> {
    deadline.check()?;
    let status = crate::commands::status_api::status_body(state, Some(deadline.remaining()?))
        .map_err(|_| "status unavailable")?;
    deadline.check()?;
    let readiness =
        crate::observability::health::readiness_body_with_budget(state, deadline.remaining()?)
            .map_err(|code| format!("readiness unavailable: {code}"))?;
    deadline.check()?;
    let (_, in_flight) = state.admission_diagnostics();
    let metrics = state.metrics().export(in_flight).into_bytes();
    deadline.check()?;
    let mut client = state
        .request_database_connection_with_budget(deadline.remaining()?)
        .map_err(|_| "events unavailable")?;
    deadline.check()?;
    lkjmc_store::pool::set_deadlines(&mut client, deadline.remaining()?)
        .map_err(|_| "events deadline unavailable")?;
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
    deadline.check()?;
    let mut output = vec![
        (
            "events.json".into(),
            redact_json(&json!({"events":events}), deadline)?,
        ),
        ("metrics.prom".into(), metrics),
        ("readiness.json".into(), redact_json(&readiness, deadline)?),
        ("status.json".into(), redact_json(&status, deadline)?),
    ];
    if let Some(log) = log::allowlisted(state, deadline)? {
        output.push(("files/daemon.log".into(), log));
    }
    deadline.check()?;
    Ok(output)
}

fn redact_json(value: &Value, deadline: Deadline) -> Result<Vec<u8>, String> {
    deadline.check()?;
    let output = crate::support::redaction::json_bytes(value)?;
    deadline.check()?;
    Ok(output)
}

fn hash(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}
