use serde::{Deserialize, Serialize};

use crate::error::IdError;
use crate::validation::{is_kebab_id, is_non_empty};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InstanceId(String);

impl InstanceId {
    pub fn parse(value: impl Into<String>) -> Result<Self, IdError> {
        let value = value.into();
        if is_kebab_id(&value) {
            Ok(Self(value))
        } else {
            Err(IdError::invalid("instance id", value))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StableId(String);

impl StableId {
    pub fn parse(kind: &'static str, value: impl Into<String>) -> Result<Self, IdError> {
        let value = value.into();
        if is_non_empty(&value) {
            Ok(Self(value))
        } else {
            Err(IdError::invalid(kind, value))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub type NodeId = StableId;
pub type JarAssetId = StableId;
pub type PlayerId = StableId;
pub type SnapshotId = StableId;
pub type CommandId = StableId;
pub type AuditEventId = StableId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Revision(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Port(pub u16);
