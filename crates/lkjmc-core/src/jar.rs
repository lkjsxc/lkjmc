use serde::{Deserialize, Serialize};

use crate::id::JarAssetId;
use crate::validation::{is_non_empty, is_sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum JarKind {
    Paper,
    Folia,
    Purpur,
    Velocity,
    Custom,
    VanillaCustom,
    ModdedCustom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JarRef(String);

impl JarRef {
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        is_non_empty(&value).then_some(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JarAsset {
    pub id: JarAssetId,
    pub kind: JarKind,
    pub project: String,
    pub channel: String,
    pub name: String,
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub source: String,
}

impl JarAsset {
    pub fn has_valid_checksum(&self) -> bool {
        is_sha256(&self.sha256)
    }

    pub fn checksum_matches(&self, actual_sha256: &str) -> bool {
        self.sha256.eq_ignore_ascii_case(actual_sha256)
    }
}
