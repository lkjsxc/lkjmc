use crate::error::IdError;
use crate::id::StableId;
use crate::validation::is_non_empty;

pub type ClaimId = StableId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimName {
    value: String,
    key: String,
}

impl ClaimName {
    pub fn parse(value: impl Into<String>) -> Result<Self, IdError> {
        let value = value.into();
        let trimmed = value.trim();
        if !is_non_empty(trimmed) || trimmed.len() > 32 {
            return Err(IdError::invalid("claim name", value));
        }
        let key = trimmed.to_ascii_lowercase();
        if !key
            .chars()
            .all(|item| item.is_ascii_alphanumeric() || item == '-' || item == '_')
        {
            return Err(IdError::invalid("claim name", value));
        }
        Ok(Self {
            value: trimmed.to_string(),
            key,
        })
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn key(&self) -> &str {
        &self.key
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClaimChunk {
    pub instance_id: String,
    pub world_name: String,
    pub chunk_x: i32,
    pub chunk_z: i32,
}

impl ClaimChunk {
    pub fn new(
        instance_id: String,
        world_name: String,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Result<Self, IdError> {
        if !is_non_empty(&instance_id) || !is_non_empty(&world_name) {
            return Err(IdError::invalid(
                "claim chunk",
                format!("{instance_id}:{world_name}"),
            ));
        }
        Ok(Self {
            instance_id,
            world_name,
            chunk_x,
            chunk_z,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimSnapshotEntry {
    pub claim_id: String,
    pub owner_uuid: String,
    pub owner_name: String,
    pub name: String,
    pub chunk: ClaimChunk,
    pub trusted_uuids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimDecision {
    Allow,
    Deny {
        claim_id: String,
        owner_name: String,
    },
}

pub fn decide(
    snapshot: &[ClaimSnapshotEntry],
    actor_uuid: &str,
    operator: bool,
    chunk: &ClaimChunk,
) -> ClaimDecision {
    if operator {
        return ClaimDecision::Allow;
    }
    let Some(entry) = snapshot.iter().find(|entry| &entry.chunk == chunk) else {
        return ClaimDecision::Allow;
    };
    if entry.owner_uuid == actor_uuid
        || entry
            .trusted_uuids
            .iter()
            .any(|trusted| trusted == actor_uuid)
    {
        return ClaimDecision::Allow;
    }
    ClaimDecision::Deny {
        claim_id: entry.claim_id.clone(),
        owner_name: entry.owner_name.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_claim_names() -> Result<(), IdError> {
        let name = ClaimName::parse("Home_Base")?;
        assert_eq!(name.value(), "Home_Base");
        assert_eq!(name.key(), "home_base");
        assert!(ClaimName::parse("bad name").is_err());
        assert!(ClaimName::parse("").is_err());
        Ok(())
    }

    #[test]
    fn decides_access_from_snapshot() -> Result<(), IdError> {
        let chunk = ClaimChunk::new("survival".to_string(), "world".to_string(), 1, 2)?;
        let snapshot = vec![ClaimSnapshotEntry {
            claim_id: "claim".to_string(),
            owner_uuid: "owner".to_string(),
            owner_name: "Owner".to_string(),
            name: "base".to_string(),
            chunk: chunk.clone(),
            trusted_uuids: vec!["friend".to_string()],
        }];
        assert_eq!(
            decide(&snapshot, "owner", false, &chunk),
            ClaimDecision::Allow
        );
        assert_eq!(
            decide(&snapshot, "friend", false, &chunk),
            ClaimDecision::Allow
        );
        assert_eq!(
            decide(&snapshot, "stranger", true, &chunk),
            ClaimDecision::Allow
        );
        assert!(matches!(
            decide(&snapshot, "stranger", false, &chunk),
            ClaimDecision::Deny { .. }
        ));
        Ok(())
    }
}
