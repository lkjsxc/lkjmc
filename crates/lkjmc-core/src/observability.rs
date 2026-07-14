use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use uuid::Uuid;

mod validation;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Component {
    Daemon,
    Cli,
    Web,
    Discord,
    Runtime,
    Network,
    Sync,
    Jvm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    CommandCompleted,
    AdmissionRejected,
    DatabaseDiagnostic,
    RuntimeDiagnostic,
    NetworkDiagnostic,
    SyncDiagnostic,
    JvmDiagnostic,
    DiscordDiagnostic,
    ReadinessChecked,
    SupportBundleCreated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Surface {
    Cli,
    Web,
    Discord,
    Paper,
    Velocity,
    Http,
    Unix,
    Internal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Succeeded,
    Failed,
    Denied,
    Cancelled,
    Degraded,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EventEnvelope {
    pub event_id: Uuid,
    pub timestamp: String,
    pub severity: Severity,
    pub component: Component,
    pub event_kind: EventKind,
    pub request_id: Option<String>,
    pub operation_id: Option<Uuid>,
    pub correlation_id: Option<Uuid>,
    pub actor_kind: String,
    pub actor_name: String,
    pub surface: Surface,
    pub outcome: Outcome,
    pub error_class: Option<String>,
    pub attributes: BTreeMap<String, Value>,
    pub source: String,
}

impl EventEnvelope {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        severity: Severity,
        component: Component,
        event_kind: EventKind,
        request_id: Option<String>,
        operation_id: Option<Uuid>,
        correlation_id: Option<Uuid>,
        actor_kind: impl Into<String>,
        actor_name: impl Into<String>,
        surface: Surface,
        outcome: Outcome,
        error_class: Option<String>,
        attributes: BTreeMap<String, Value>,
        source: impl Into<String>,
    ) -> Result<Self, String> {
        Self::with_event_id(
            Uuid::new_v4(),
            severity,
            component,
            event_kind,
            request_id,
            operation_id,
            correlation_id,
            actor_kind,
            actor_name,
            surface,
            outcome,
            error_class,
            attributes,
            source,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_event_id(
        event_id: Uuid,
        severity: Severity,
        component: Component,
        event_kind: EventKind,
        request_id: Option<String>,
        operation_id: Option<Uuid>,
        correlation_id: Option<Uuid>,
        actor_kind: impl Into<String>,
        actor_name: impl Into<String>,
        surface: Surface,
        outcome: Outcome,
        error_class: Option<String>,
        attributes: BTreeMap<String, Value>,
        source: impl Into<String>,
    ) -> Result<Self, String> {
        let (mut attributes, mut redacted) = validation::sanitize_attributes(attributes);
        let (actor_kind, changed) = validation::required(actor_kind.into(), 32);
        redacted |= changed;
        let (actor_name, changed) = validation::required(actor_name.into(), 96);
        redacted |= changed;
        let (request_id, changed) = validation::optional(request_id, 128);
        redacted |= changed;
        let (error_class, changed) = validation::optional(error_class, 64);
        redacted |= changed;
        let (source, changed) = validation::required(source.into(), 64);
        redacted |= changed;
        if redacted {
            attributes.insert("redacted".into(), Value::Bool(true));
        }
        Ok(Self {
            event_id,
            timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            severity,
            component,
            event_kind,
            request_id,
            operation_id,
            correlation_id,
            actor_kind,
            actor_name,
            surface,
            outcome,
            error_class,
            attributes,
            source,
        })
    }
}

pub fn correlation_ids(body: &Value, execution_id: Uuid) -> (Uuid, Uuid) {
    let operation = body
        .get("operationId")
        .and_then(Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
        .unwrap_or(execution_id);
    let correlation = body
        .get("correlationId")
        .and_then(Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
        .unwrap_or(operation);
    (operation, correlation)
}
