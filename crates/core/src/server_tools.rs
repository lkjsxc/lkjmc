//! Server explorer, passive job bounds, and explicit native operator intent.
use crate::{
    App,
    auth::Actor,
    commands::{Command, job},
    error::{Error, Result},
    hosting,
};
use axum::{
    Json,
    extract::{Path, State},
};
use serde_json::{Value, json};
use sqlx::PgConnection;
use uuid::Uuid;

pub fn passive(kind: &str) -> bool {
    matches!(kind, "server.logs" | "server.files" | "server.file.read")
}
pub fn world_data(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or("");
    matches!(name, "level.dat" | "level.dat_old" | "session.lock")
        || [".dat", ".mca", ".mcr"].iter().any(|s| name.ends_with(s))
}
pub fn protected(path: &str) -> bool {
    path.split('/').any(|p| {
        let p = p.to_ascii_lowercase();
        p.starts_with("lkjmc-")
            || p.starts_with('.')
            || matches!(
                p.as_str(),
                "eula.txt"
                    | "server.properties"
                    | "ops.json"
                    | "whitelist.json"
                    | "usercache.json"
                    | "banned-players.json"
                    | "banned-ips.json"
                    | "config"
                    | "plugins"
                    | "logs"
                    | "crash-reports"
                    | "permissions.json"
                    | "paper.yml"
                    | "spigot.yml"
                    | "bukkit.yml"
                    | "velocity.toml"
            )
            || [
                "secret",
                "credential",
                "password",
                "token",
                "private",
                "session",
                "auth",
            ]
            .iter()
            .any(|v| p.contains(v))
            || [".pem", ".key", ".p12", ".keystore", ".env"]
                .iter()
                .any(|v| p.ends_with(v))
    })
}
fn hash(value: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err(Error::invalid(
            "Use the SHA-256 returned by the latest file read.",
        ));
    }
    Ok(())
}
pub fn date(value: &str) -> Result<()> {
    if value.len() != 10
        || chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .ok()
            .is_none_or(|d| d.format("%Y-%m-%d").to_string() != value)
    {
        return Err(Error::invalid(
            "Choose a valid UTC date in YYYY-MM-DD format.",
        ));
    }
    Ok(())
}
pub async fn identity(
    db: &mut PgConnection,
    server: Uuid,
    member: Uuid,
    grant: bool,
) -> Result<Value> {
    let rows: Vec<Value> = sqlx::query_scalar("SELECT jsonb_build_object('uuid',i.subject,'name',i.display_name) FROM identities i JOIN accounts a ON a.id=i.account_id JOIN profiles p ON p.account_id=a.id AND p.status='active' JOIN servers s ON s.id=$1 WHERE i.account_id=$2 AND i.issuer='java' AND i.subject=p.native_uuid::text AND a.merged_into IS NULL AND (a.banned_until IS NULL OR a.banned_until<now()) AND (NOT $3 OR s.owner=a.id OR EXISTS(SELECT 1 FROM server_members m WHERE m.server_id=s.id AND m.account_id=a.id)) AND (SELECT count(*) FROM identities x WHERE x.account_id=a.id AND x.issuer='java')=1")
        .bind(server).bind(member).bind(grant).fetch_all(&mut *db).await?;
    let value=rows.first().filter(|_|rows.len()==1).ok_or_else(|| Error::conflict("The member needs one verified Java identity matching their active native UUID. Unlinked or ambiguous identities cannot receive Minecraft OP."))?;
    let name = value["name"].as_str().unwrap_or("");
    if !(3..=16).contains(&name.len())
        || !name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
        || value["uuid"]
            .as_str()
            .and_then(|s| s.parse::<Uuid>().ok())
            .is_none()
    {
        return Err(Error::conflict(
            "The verified Minecraft identity is invalid.",
        ));
    }
    Ok(value.clone())
}
pub async fn stopped(db: &mut PgConnection, id: Uuid, op: bool) -> Result<()> {
    let valid: bool=sqlx::query_scalar("SELECT kind='custom' AND desired='stopped' AND observed='stopped' AND (NOT $2 OR software='paper') FROM servers WHERE id=$1 FOR UPDATE")
        .bind(id).bind(op).fetch_optional(&mut *db).await?.ok_or_else(Error::missing)?;
    if !valid {
        return Err(Error::conflict(if op {
            "Minecraft OP requires a stopped custom Paper server; it takes effect at the next start."
        } else {
            "Stop the custom game server before changing files."
        }));
    }
    Ok(())
}
pub async fn command(db: &mut PgConnection, actor: &Actor, command: &Command) -> Result<Value> {
    use Command::*;
    let (id, kind, payload) = match command {
        ServerLogs { id, date: d } => {
            if let Some(d) = d {
                date(d)?;
            }
            (*id, "server.logs", json!({"date":d}))
        }
        ServerFiles { id, path } => {
            if !path.is_empty() {
                hosting::safe_path(path)?;
            }
            (*id, "server.files", json!({"path":path}))
        }
        ServerFileRead { id, path } => {
            hosting::safe_path(path)?;
            (*id, "server.file.read", json!({"path":path}))
        }
        ServerFileWrite {
            id,
            path,
            text,
            expected_sha256,
        } => {
            hosting::safe_path(path)?;
            if world_data(path) {
                return Err(Error::invalid(
                    "Individual world data files are protected. Use a complete world archive.",
                ));
            }
            if text.len() > 65536 || text.contains('\0') {
                return Err(Error::invalid(
                    "Text must be UTF-8, contain no NUL, and be at most 64 KiB.",
                ));
            }
            if let Some(h) = expected_sha256 {
                hash(h)?;
            }
            (
                *id,
                "server.file.write",
                json!({"path":path,"text":text,"expected_sha256":expected_sha256}),
            )
        }
        ServerFileDelete {
            id,
            path,
            expected_sha256,
        } => {
            hosting::safe_path(path)?;
            if world_data(path) {
                return Err(Error::invalid(
                    "Individual world data files are protected. Use a complete world archive.",
                ));
            }
            hash(expected_sha256)?;
            (
                *id,
                "server.file.delete",
                json!({"path":path,"expected_sha256":expected_sha256}),
            )
        }
        ServerDirectoryCreate { id, path } => {
            hosting::safe_path(path)?;
            (*id, "server.directory.create", json!({"path":path}))
        }
        ServerOperator {
            id,
            member,
            operator,
        } => {
            hosting::server_permission(db, actor.id, *id, true).await?;
            stopped(db, *id, true).await?;
            let native = identity(db, *id, *member, *operator).await?;
            (
                *id,
                "server.operator",
                json!({"member":member,"operator":operator,"identity":native}),
            )
        }
        _ => return Err(Error::invalid("Unsupported server tool.")),
    };
    hosting::server_permission(db, actor.id, id, kind != "server.logs").await?;
    if !passive(kind) {
        stopped(db, id, kind == "server.operator").await?;
        return job(db, actor.id, Some(id), "host", kind, payload).await;
    }
    // Serialize admission globally: a polling tab cannot create unbounded queued work.
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended('host-passive-admission',0))")
        .execute(&mut *db)
        .await?;
    prune(db).await?;
    if let Some(v)=sqlx::query_scalar::<_,Value>("SELECT jsonb_build_object('job_id',id,'state',state) FROM jobs WHERE actor=$1 AND server_id=$2 AND kind=$3 AND payload=$4 AND state IN ('queued','leased','waiting') ORDER BY created_at LIMIT 1")
        .bind(actor.id).bind(id).bind(kind).bind(&payload).fetch_optional(&mut *db).await? {return Ok(v);}
    let busy:bool=sqlx::query_scalar("SELECT count(*)>=32 OR count(*) FILTER(WHERE actor=$1)>=3 FROM jobs WHERE kind IN ('server.logs','server.files','server.file.read') AND state IN ('queued','leased','waiting')").bind(actor.id).fetch_one(&mut *db).await?;
    if busy {
        return Err(Error::conflict(
            "Server reads are busy. Wait for the current read to finish.",
        ));
    }
    job(db, actor.id, Some(id), "host", kind, payload).await
}
pub async fn prune(db: &mut PgConnection) -> Result<()> {
    sqlx::query("DELETE FROM jobs WHERE kind IN ('server.logs','server.files','server.file.read') AND (state IN ('succeeded','failed','cancelled') AND (updated_at<now()-interval '2 minutes' OR id IN (SELECT id FROM jobs WHERE kind IN ('server.logs','server.files','server.file.read') AND state IN ('succeeded','failed','cancelled') ORDER BY updated_at DESC OFFSET 128)) OR state IN ('queued','waiting') AND created_at<now()-interval '2 minutes' OR state='leased' AND lease_until<now() AND created_at<now()-interval '2 minutes')").execute(db).await?;
    Ok(())
}
pub async fn read_job(
    State(app): State<App>,
    actor: Actor,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>> {
    // Permission and result share one database snapshot, including revocation.
    let row:Option<(Value,bool)>=sqlx::query_as("SELECT jsonb_build_object('id',j.id,'kind',j.kind,'server_id',j.server_id,'state',j.state,'progress',j.progress,'result',j.result,'error',j.error,'updated_at',j.updated_at), j.kind NOT IN ('server.logs','server.files','server.file.read') OR (j.updated_at>now()-interval '2 minutes' AND EXISTS(SELECT 1 FROM servers s JOIN accounts a ON a.id=$2 WHERE s.id=j.server_id AND a.merged_into IS NULL AND (a.banned_until IS NULL OR a.banned_until<now()) AND (s.owner=a.id OR a.administrator OR EXISTS(SELECT 1 FROM server_members m WHERE m.server_id=s.id AND m.account_id=a.id AND (m.role='administrator' OR j.kind='server.logs' AND m.role='operator')) OR EXISTS(SELECT 1 FROM community_members m WHERE m.community_id=s.community_id AND m.account_id=a.id AND m.administrator)))) FROM jobs j WHERE j.id=$1 AND (j.actor=$2 OR $3)").bind(id).bind(actor.id).bind(actor.admin).fetch_optional(&app.db).await?;
    let (value, allowed) = row.ok_or_else(Error::missing)?;
    if !allowed {
        return Err(Error::forbidden());
    }
    Ok(Json(value))
}

/// Host dispatch keeps passive reads behind all real effects. Other workers retain their dispatcher.
pub async fn poll(
    State(app): State<App>,
    service: crate::services::Service,
) -> Result<Json<Value>> {
    if service.role != "host" {
        return crate::services::poll(State(app), service).await;
    }
    let mut tx = app.db.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended('host-job-dispatch',0))")
        .execute(&mut *tx)
        .await?;
    prune(&mut tx).await?;
    let id: Option<Uuid> = sqlx::query_scalar("SELECT j.id FROM jobs j JOIN servers s ON s.id=j.server_id WHERE j.worker='host' AND (j.state='queued' OR j.state='waiting' AND j.updated_at<now()-interval '5 seconds' OR j.state='leased' AND j.lease_until<now()) AND (s.maintenance_job_id IS NULL OR s.maintenance_job_id=j.id) AND NOT EXISTS(SELECT 1 FROM jobs other WHERE other.id<>j.id AND other.worker='host' AND other.server_id=j.server_id AND other.state='leased' AND other.lease_until>now()) ORDER BY CASE WHEN j.kind IN ('server.logs','server.files','server.file.read') THEN 2 WHEN j.kind='official.backup.prune' THEN 1 ELSE 0 END,j.updated_at,j.created_at FOR UPDATE OF j SKIP LOCKED LIMIT 1").fetch_optional(&mut *tx).await?;
    let Some(id) = id else {
        tx.commit().await?;
        return Ok(Json(json!({"job":null})));
    };
    let value:Value=sqlx::query_scalar("UPDATE jobs SET state='leased',lease_token=$2,lease_owner=$3,lease_until=now()+interval '90 seconds',attempts=attempts+1,updated_at=now() WHERE id=$1 RETURNING to_jsonb(jobs)").bind(id).bind(Uuid::new_v4()).bind(service.id).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(json!({"job":value})))
}

#[derive(serde::Deserialize)]
pub struct ToolAck {
    lease_token: Uuid,
    state: String,
    #[serde(default)]
    progress: Value,
    #[serde(default)]
    result: Value,
    error: Option<String>,
}
fn tool_effect(kind: &str) -> bool {
    matches!(
        kind,
        "server.file.write" | "server.file.delete" | "server.directory.create" | "server.operator"
    )
}
fn validate_result(kind: &str, payload: &Value, result: &Value) -> Result<()> {
    use sha2::{Digest, Sha256};
    let invalid =
        || Error::invalid("The guest result does not verify the requested server tool effect.");
    if serde_json::to_vec(result).map_err(Error::internal)?.len() > 2 * 1024 * 1024 {
        return Err(invalid());
    }
    match kind {
        "server.logs" => {
            let lines = result["lines"].as_array().ok_or_else(invalid)?;
            if lines.len() > 200
                || lines
                    .iter()
                    .any(|v| v.as_str().is_none_or(|s| s.chars().count() > 1024))
                || result["date"] != payload["date"]
            {
                return Err(invalid());
            }
            if let Some(dates) = result.get("dates") {
                let dates = dates.as_array().ok_or_else(invalid)?;
                if dates.len() > 366 {
                    return Err(invalid());
                }
                for d in dates {
                    date(d.as_str().ok_or_else(invalid)?)?;
                }
            }
        }
        "server.files" => {
            if result["path"] != payload["path"] {
                return Err(invalid());
            }
            let entries = result["entries"].as_array().ok_or_else(invalid)?;
            if entries.len() > 256 {
                return Err(invalid());
            }
            for e in entries {
                let p = e["path"].as_str().ok_or_else(invalid)?;
                hosting::safe_path(p)?;
                let name = e["name"].as_str().ok_or_else(invalid)?;
                let parent = payload["path"].as_str().ok_or_else(invalid)?;
                if name.contains('/')
                    || p != if parent.is_empty() {
                        name.to_string()
                    } else {
                        format!("{parent}/{name}")
                    }
                    || !matches!(e["kind"].as_str(), Some("file" | "directory"))
                {
                    return Err(invalid());
                }
            }
        }
        "server.file.read" => {
            let text = result["text"].as_str().ok_or_else(invalid)?;
            if text.len() > 65536
                || text.contains('\0')
                || result["path"] != payload["path"]
                || result["bytes"].as_u64() != Some(text.len() as u64)
                || result["sha256"] != hex::encode(Sha256::digest(text.as_bytes()))
            {
                return Err(invalid());
            }
        }
        "server.file.write" => {
            if result["effect"] != "committed"
                || result["path"] != payload["path"]
                || result["sha256"]
                    != hex::encode(Sha256::digest(
                        payload["text"].as_str().ok_or_else(invalid)?.as_bytes(),
                    ))
            {
                return Err(invalid());
            }
        }
        "server.file.delete" | "server.directory.create" => {
            if result["effect"] != "committed" || result["path"] != payload["path"] {
                return Err(invalid());
            }
        }
        "server.operator" => {
            if result["effect"] != "committed"
                || result["member"] != payload["member"]
                || result["operator"] != payload["operator"]
                || result["native_uuid"] != payload["identity"]["uuid"]
                || result["effective"] != "next_start"
            {
                return Err(invalid());
            }
            hash(result["sha256"].as_str().ok_or_else(invalid)?)?;
        }
        _ => return Err(invalid()),
    }
    Ok(())
}

/// Dedicated settlement for these new kinds; all other kinds keep existing settlement.
/// This avoids changing the coordinator-owned shared acknowledgement/notification code.
pub async fn ack(
    State(app): State<App>,
    service: crate::services::Service,
    Path(id): Path<Uuid>,
    Json(value): Json<Value>,
) -> Result<Json<Value>> {
    use sqlx::Row;
    let request: ToolAck = serde_json::from_value(value.clone())
        .map_err(|_| Error::invalid("Invalid acknowledgement."))?;
    let kind: Option<String> = sqlx::query_scalar(
        "SELECT kind FROM jobs WHERE id=$1 AND lease_owner=$2 AND lease_token=$3",
    )
    .bind(id)
    .bind(service.id)
    .bind(request.lease_token)
    .fetch_optional(&app.db)
    .await?;
    let kind =
        kind.ok_or_else(|| Error::conflict("The job lease changed or this bounded read expired."))?;
    if kind == "server.install" && request.state == "succeeded" {
        let expected: Value = sqlx::query_scalar("SELECT payload->'path' FROM jobs WHERE id=$1")
            .bind(id)
            .fetch_one(&app.db)
            .await?;
        if request.result["path"] != expected {
            return Err(Error::invalid(
                "The installed artifact destination does not match the requested exact path.",
            ));
        }
    }
    if !passive(&kind) && !tool_effect(&kind) {
        return crate::services::ack(
            State(app),
            service,
            Path(id),
            Json(serde_json::from_value(value).map_err(Error::internal)?),
        )
        .await;
    }
    service.require("host")?;
    let mut tx = app.db.begin().await?;
    let row = sqlx::query(
        "SELECT * FROM jobs WHERE id=$1 AND lease_owner=$2 AND lease_token=$3 FOR UPDATE",
    )
    .bind(id)
    .bind(service.id)
    .bind(request.lease_token)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| Error::conflict("The job lease changed."))?;
    let old: String = row.get("state");
    if matches!(old.as_str(), "succeeded" | "failed" | "cancelled") {
        if matches!(request.state.as_str(), "succeeded" | "failed")
            && (old != request.state || row.get::<Value, _>("result") != request.result)
        {
            return Err(Error::conflict(
                "The committed receipt changed; reconcile this job.",
            ));
        }
        return Ok(Json(json!({"id":id,"state":old})));
    }
    if old != "leased"
        || !matches!(
            request.state.as_str(),
            "leased" | "waiting" | "succeeded" | "failed"
        )
    {
        return Err(Error::conflict("This job cannot be updated right now."));
    }
    if request.state == "succeeded" {
        if tool_effect(&kind)
            && row
                .get::<Option<chrono::DateTime<chrono::Utc>>, _>("host_authorized_at")
                .is_none()
        {
            return Err(Error::forbidden());
        }
        validate_result(&kind, &row.get::<Value, _>("payload"), &request.result)?;
    }
    if request.state == "failed"
        && !matches!(
            request.result["effect"].as_str(),
            Some("none" | "rolled_back")
        )
    {
        return Err(Error::conflict(
            "Recover the guest receipt before declaring failure.",
        ));
    }
    let error = request
        .error
        .map(|s| s.chars().take(2000).collect::<String>());
    sqlx::query("UPDATE jobs SET state=$2,progress=CASE WHEN $2='leased' AND $3='{}'::jsonb THEN progress ELSE $3 END,result=CASE WHEN $2 IN ('succeeded','failed') THEN $4 ELSE result END,error=$5,lease_until=CASE WHEN $2='leased' THEN now()+interval '90 seconds' ELSE NULL END,updated_at=now() WHERE id=$1").bind(id).bind(&request.state).bind(request.progress).bind(request.result).bind(error).execute(&mut *tx).await?;
    if matches!(request.state.as_str(), "succeeded" | "failed") {
        sqlx::query("UPDATE servers s SET maintenance=EXISTS(SELECT 1 FROM jobs j WHERE j.server_id=s.id AND j.id<>$1 AND j.kind='server.stop' AND j.state IN ('queued','leased','waiting')),maintenance_job_id=NULL WHERE maintenance_job_id=$1").bind(id).execute(&mut *tx).await?;
        if !passive(&kind) {
            crate::commands::notify(
                &mut tx,
                row.get("actor"),
                "job_finished",
                json!({"id":id,"kind":kind,"state":request.state}),
            )
            .await?;
        }
    }
    tx.commit().await?;
    Ok(Json(json!({"id":id,"state":request.state})))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_dates_protected_paths_and_receipt_digests() {
        for bad in ["2026-2-03", "2026-02-30", "--help", ""] {
            assert!(date(bad).is_err());
        }
        assert!(date("2024-02-29").is_ok());
        for bad in [
            "config/paper-global.yml",
            "server.properties",
            "ops.json",
            "mods/secret.txt",
            "plugins/a.jar",
        ] {
            assert!(hosting::safe_path(bad).is_err());
        }
        assert!(hosting::safe_path("world/datapacks/example.zip").is_ok());
        assert!(
            validate_result(
                "server.file.write",
                &json!({"path":"a.txt","text":"hello"}),
                &json!({"effect":"committed","path":"a.txt","sha256":"bad"})
            )
            .is_err()
        );
        assert!(validate_result("server.operator",&json!({"member":"m","operator":true,"identity":{"uuid":"u"}}),&json!({"effect":"committed","member":"m","operator":true,"native_uuid":"other","effective":"next_start","sha256":"a".repeat(64)})).is_err());
    }
}
