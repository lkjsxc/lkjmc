use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::id::{PlayerId, Revision, SnapshotId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileLease {
    pub player_uuid: PlayerId,
    pub scope: String,
    pub holder: String,
    pub revision: Revision,
    pub expires_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileSnapshot {
    pub id: SnapshotId,
    pub player_uuid: PlayerId,
    pub scope: String,
    pub revision: Revision,
    pub payload_format: String,
    pub payload_sha256: String,
    pub source_instance: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerSettings {
    pub player_uuid: PlayerId,
    pub language: String,
    pub menu_enabled: bool,
    pub hud_enabled: bool,
}
