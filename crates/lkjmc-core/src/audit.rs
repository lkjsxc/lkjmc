use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::command::ActorKind;
use crate::id::AuditEventId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditResult {
    Requested,
    Succeeded,
    Failed,
    Denied,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEvent {
    pub id: AuditEventId,
    pub actor_kind: ActorKind,
    pub actor_name: String,
    pub action: String,
    pub target_kind: String,
    pub target_id: String,
    pub result: AuditResult,
    pub metadata: Value,
}
