use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewClaim<'a> {
    pub id: Uuid,
    pub owner_uuid: Uuid,
    pub owner_name: &'a str,
    pub name: &'a str,
    pub instance_id: &'a str,
    pub world_name: &'a str,
    pub chunk_x: i32,
    pub chunk_z: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimSummary {
    pub id: Uuid,
    pub owner_uuid: Uuid,
    pub owner_name: String,
    pub name: String,
    pub chunk_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimChunkRecord {
    pub claim_id: Uuid,
    pub owner_uuid: Uuid,
    pub owner_name: String,
    pub name: String,
    pub instance_id: String,
    pub world_name: String,
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub trusts: Vec<TrustedPlayer>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedPlayer {
    pub trusted_uuid: Uuid,
    pub trusted_name: String,
}
