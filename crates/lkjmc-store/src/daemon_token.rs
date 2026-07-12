use postgres::GenericClient;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonTokenRecord {
    pub credential_id: Uuid,
    pub surface: String,
    pub principal_kind: String,
    pub principal_id: String,
    pub scopes: Vec<String>,
    pub expires_at_seconds: i64,
}

#[allow(clippy::too_many_arguments)]
pub fn insert(
    client: &mut impl GenericClient,
    credential_id: Uuid,
    token_hash: &str,
    surface: &str,
    principal_kind: &str,
    principal_id: &str,
    scopes: &[String],
    expiry_seconds: i64,
) -> Result<(), StoreError> {
    client.execute(
        "insert into daemon_tokens
         (credential_id, token_hash, surface, principal_kind, principal_id, scopes, expires_at)
         values ($1, $2, $3, $4, $5, $6, now() + ($7 * interval '1 second'))",
        &[
            &credential_id,
            &token_hash,
            &surface,
            &principal_kind,
            &principal_id,
            &scopes,
            &expiry_seconds,
        ],
    )?;
    Ok(())
}

pub fn find_active(
    client: &mut impl GenericClient,
    token_hash: &str,
) -> Result<Option<DaemonTokenRecord>, StoreError> {
    let row = client.query_opt(
        "update daemon_tokens set last_used_at = now()
         where token_hash = $1 and revoked_at is null and expires_at > now()
         returning credential_id, surface, principal_kind, principal_id, scopes,
                   extract(epoch from expires_at)::bigint",
        &[&token_hash],
    )?;
    Ok(row.map(|row| DaemonTokenRecord {
        credential_id: row.get(0),
        surface: row.get(1),
        principal_kind: row.get(2),
        principal_id: row.get(3),
        scopes: row.get(4),
        expires_at_seconds: row.get(5),
    }))
}

pub fn current_revision(client: &mut impl GenericClient) -> Result<i64, StoreError> {
    Ok(client
        .query_one(
            "select revision from daemon_token_revision where singleton = true",
            &[],
        )?
        .get(0))
}

pub fn revoke(client: &mut impl GenericClient, credential_id: Uuid) -> Result<u64, StoreError> {
    Ok(client.execute("update daemon_tokens set revoked_at = now() where credential_id = $1 and revoked_at is null", &[&credential_id])?)
}

pub fn active_count(client: &mut impl GenericClient) -> Result<i64, StoreError> {
    Ok(client.query_one("select count(*)::bigint from daemon_tokens where revoked_at is null and expires_at > now()", &[])?.get(0))
}
