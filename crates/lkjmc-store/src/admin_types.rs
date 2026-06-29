use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminGrant {
    pub id: Uuid,
    pub principal_kind: String,
    pub principal_id: String,
    pub role_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAuditRow {
    pub actor_kind: String,
    pub actor_id: String,
    pub action: String,
    pub target_kind: Option<String>,
    pub target_id: Option<String>,
    pub result: String,
}
