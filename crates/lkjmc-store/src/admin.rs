use lkjmc_core::admin::AdminRole;
use postgres::Client;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::admin_types::{AdminAuditRow, AdminGrant};
use crate::error::StoreError;

pub fn upsert_builtin_roles(client: &mut Client) -> Result<(), StoreError> {
    for role in AdminRole::all() {
        client.execute(
            "insert into admin_roles (id, title_key, permissions)
             values ($1, $2, $3)
             on conflict (id) do update set title_key = excluded.title_key,
             permissions = excluded.permissions",
            &[
                &role.id(),
                &format!("admin.role.{}", role.id()),
                &json!(role.permissions()),
            ],
        )?;
    }
    Ok(())
}

pub fn grant_role(
    client: &mut Client,
    principal_kind: &str,
    principal_id: &str,
    role_id: &str,
    reason: &str,
    actor_kind: &str,
    actor_id: &str,
) -> Result<Uuid, StoreError> {
    upsert_builtin_roles(client)?;
    let id = Uuid::new_v4();
    client.execute(
        "insert into admin_grants
         (id, principal_kind, principal_id, role_id, reason, granted_by_kind, granted_by_id)
         values ($1, $2, $3, $4, $5, $6, $7)",
        &[
            &id,
            &principal_kind,
            &principal_id,
            &role_id,
            &reason,
            &actor_kind,
            &actor_id,
        ],
    )?;
    insert_audit(
        client,
        actor_kind,
        actor_id,
        "admin.grant.create",
        principal_kind,
        principal_id,
        "ok",
    )?;
    Ok(id)
}

pub fn revoke_grants(
    client: &mut Client,
    principal_kind: &str,
    principal_id: &str,
    role_id: &str,
    reason: &str,
    actor_kind: &str,
    actor_id: &str,
) -> Result<u64, StoreError> {
    let count = client.execute(
        "update admin_grants set revoked_at = now(), revoked_by_kind = $4,
         revoked_by_id = $5, revoke_reason = $6
         where principal_kind = $1 and principal_id = $2 and role_id = $3 and revoked_at is null",
        &[
            &principal_kind,
            &principal_id,
            &role_id,
            &actor_kind,
            &actor_id,
            &reason,
        ],
    )?;
    insert_audit(
        client,
        actor_kind,
        actor_id,
        "admin.grant.revoke",
        principal_kind,
        principal_id,
        "ok",
    )?;
    Ok(count)
}

pub fn list_grants(
    client: &mut Client,
    kind: &str,
    id: &str,
) -> Result<Vec<AdminGrant>, StoreError> {
    upsert_builtin_roles(client)?;
    let rows = client.query(
        "select id, principal_kind, principal_id, role_id, reason
         from admin_grants where principal_kind = $1 and principal_id = $2
         and revoked_at is null and (expires_at is null or expires_at > now())
         order by created_at desc",
        &[&kind, &id],
    )?;
    Ok(rows.into_iter().map(grant_from_row).collect())
}

pub fn effective_permissions(
    client: &mut Client,
    kind: &str,
    id: &str,
) -> Result<Vec<String>, StoreError> {
    let rows = client.query(
        "select distinct jsonb_array_elements_text(role.permissions)
         from admin_grants g join admin_roles role on role.id = g.role_id
         where g.principal_kind = $1 and g.principal_id = $2
         and g.revoked_at is null and (g.expires_at is null or g.expires_at > now())
         order by 1",
        &[&kind, &id],
    )?;
    Ok(rows.into_iter().map(|row| row.get(0)).collect())
}

pub fn tail_audit(client: &mut Client, limit: i64) -> Result<Vec<AdminAuditRow>, StoreError> {
    let rows = client.query(
        "select actor_kind, actor_id, action, target_kind, target_id, result
         from admin_audit order by created_at desc limit $1",
        &[&limit],
    )?;
    Ok(rows.into_iter().map(audit_from_row).collect())
}

pub fn insert_audit(
    client: &mut Client,
    actor_kind: &str,
    actor_id: &str,
    action: &str,
    target_kind: &str,
    target_id: &str,
    result: &str,
) -> Result<(), StoreError> {
    client.execute(
        "insert into admin_audit
         (id, actor_kind, actor_id, action, target_kind, target_id, result, metadata)
         values ($1, $2, $3, $4, $5, $6, $7, $8)",
        &[
            &Uuid::new_v4(),
            &actor_kind,
            &actor_id,
            &action,
            &target_kind,
            &target_id,
            &result,
            &Value::Object(Default::default()),
        ],
    )?;
    Ok(())
}

fn grant_from_row(row: postgres::Row) -> AdminGrant {
    AdminGrant {
        id: row.get(0),
        principal_kind: row.get(1),
        principal_id: row.get(2),
        role_id: row.get(3),
        reason: row.get(4),
    }
}

fn audit_from_row(row: postgres::Row) -> AdminAuditRow {
    AdminAuditRow {
        actor_kind: row.get(0),
        actor_id: row.get(1),
        action: row.get(2),
        target_kind: row.get(3),
        target_id: row.get(4),
        result: row.get(5),
    }
}
