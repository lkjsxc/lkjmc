use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use uuid::Uuid;

const ID_NAMESPACE: Uuid = Uuid::from_u128(0x1178c90e_4490_4c95_b713_760f4987dce1);
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
        validation::attributes(&attributes)?;
        let actor_kind = validation::bounded("actorKind", actor_kind.into(), 32)?;
        let actor_name = validation::bounded("actorName", actor_name.into(), 96)?;
        let request_id = validation::optional_bounded("requestId", request_id, 128)?;
        let error_class = validation::optional_bounded("errorClass", error_class, 64)?;
        Ok(Self {
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
            source: validation::bounded("source", source.into(), 64)?,
        })
    }
}

pub fn correlation_ids(request_id: &str, body: &Value) -> (Uuid, Uuid) {
    let operation = body
        .get("operationId")
        .and_then(Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
        .unwrap_or_else(|| Uuid::new_v5(&ID_NAMESPACE, request_id.as_bytes()));
    let correlation = body
        .get("correlationId")
        .and_then(Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
        .unwrap_or(operation);
    (operation, correlation)
}
