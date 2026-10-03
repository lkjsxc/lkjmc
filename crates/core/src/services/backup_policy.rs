use super::{Service, backup, host::Lease, uuid};
use crate::{
    App,
    commands::job,
    error::{Error, Result},
};
use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{PgConnection, Row};
use std::fs::File;
use tokio::io::AsyncReadExt;
use uuid::Uuid;

pub(crate) async fn queue(
    db: &mut PgConnection,
    actor: Uuid,
    server: Uuid,
    date: Option<NaiveDate>,
) -> Result<Value> {
    sqlx::query("SELECT id FROM servers WHERE id=$1 AND kind='official' FOR UPDATE")
        .bind(server)
        .fetch_optional(&mut *db)
        .await?
        .ok_or_else(Error::missing)?;
    let busy: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM backups WHERE kind='official' AND state IN ('queued','freezing','saving','verifying','restoring'))")
        .fetch_one(&mut *db).await?;
    if busy {
        return Err(Error::conflict(
            "An official backup is already in progress.",
        ));
    }
    let backup = Uuid::new_v4();
    sqlx::query("INSERT INTO backups(id,server_id,kind,state,scheduled_for) VALUES($1,$2,'official','queued',$3)")
        .bind(backup).bind(server).bind(date).execute(&mut *db).await?;
    let queued = job(
        db,
        actor,
        Some(server),
        "host",
        "official.backup",
        json!({"backup_id":backup}),
    )
    .await?;
    sqlx::query("UPDATE backups SET job_id=$2 WHERE id=$1")
        .bind(backup)
        .bind(uuid(&queued, "job_id")?)
        .execute(db)
        .await?;
    Ok(queued)
}

/// At most the latest due daily slot is queued after downtime. Historical slots
/// are never backfilled with snapshots that would misleadingly share today's data.
pub async fn maintenance(app: &App, now: DateTime<Utc>) -> Result<()> {
    if app.config.development || !app.config.automatic_backups {
        return Ok(());
    }
    let mut tx = app.db.begin().await?;
    if !crate::deployment::permits(&mut tx).await? {
        return Ok(());
    }
    let acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock(918740022)")
        .fetch_one(&mut *tx)
        .await?;
    if !acquired {
        return Ok(());
    }
    let paused: bool = sqlx::query_scalar(
        "SELECT value='true' FROM settings WHERE key='official_mutations_paused' FOR SHARE",
    )
    .fetch_one(&mut *tx)
    .await?;
    if paused {
        return Ok(());
    }
    let Some(server) = sqlx::query_scalar::<_,Uuid>("SELECT id FROM servers WHERE kind='official' ORDER BY created_at LIMIT 1 FOR UPDATE SKIP LOCKED")
        .fetch_optional(&mut *tx).await? else { return Ok(()); };
    let date = (now - Duration::hours(i64::from(app.config.backup_hour_utc))).date_naive();
    let due: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM backups WHERE server_id=$1 AND scheduled_for=$2) AND NOT EXISTS(SELECT 1 FROM backups WHERE kind='official' AND state IN ('queued','freezing','saving','verifying','restoring'))")
        .bind(server).bind(date).fetch_one(&mut *tx).await?;
    if due {
        if let Some(actor) = sqlx::query_scalar::<_,Uuid>("SELECT id FROM accounts WHERE administrator AND merged_into IS NULL AND (banned_until IS NULL OR banned_until<now()) ORDER BY created_at LIMIT 1")
            .fetch_optional(&mut *tx).await? {
            queue(&mut tx, actor, server, Some(date)).await?;
        }
    }
    // Keep successful backups for seven distinct UTC dates and four distinct
    // ISO weeks, even across outages. A failed/unfinished backup displaces none.
    // Manual and pinned backups are retained independently of these generations.
    let obsolete = sqlx::query(r#"
        WITH eligible AS (
            SELECT id,completed_at FROM backups WHERE server_id=$1 AND kind='official'
            AND state='ready' AND scheduled_for IS NOT NULL AND completed_at IS NOT NULL
            AND manifest->'verified'='true'::jsonb
        ), daily AS (
            SELECT DISTINCT ON ((completed_at AT TIME ZONE 'UTC')::date) id,completed_at
            FROM eligible ORDER BY (completed_at AT TIME ZONE 'UTC')::date DESC,completed_at DESC,id DESC LIMIT 7
        ), weekly AS (
            SELECT DISTINCT ON (date_trunc('week',completed_at AT TIME ZONE 'UTC')) id,completed_at
            FROM eligible ORDER BY date_trunc('week',completed_at AT TIME ZONE 'UTC') DESC,completed_at DESC,id DESC LIMIT 4
        ) SELECT b.id,j.actor FROM backups b JOIN jobs j ON j.id=b.job_id
        WHERE b.id IN (SELECT id FROM eligible) AND NOT b.pinned
        AND b.id NOT IN (SELECT id FROM daily UNION SELECT id FROM weekly)
        ORDER BY b.completed_at LIMIT 20 FOR UPDATE OF b SKIP LOCKED
    "#).bind(server).fetch_all(&mut *tx).await?;
    for row in obsolete {
        let id: Uuid = row.get("id");
        let queued = job(
            &mut tx,
            row.get("actor"),
            Some(server),
            "host",
            "official.backup.prune",
            json!({"backup_id":id}),
        )
        .await?;
        sqlx::query("UPDATE backups SET state='pruning',prune_job_id=$2 WHERE id=$1")
            .bind(id)
            .bind(uuid(&queued, "job_id")?)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Automatic work is authorized by its durable schedule/retention record,
/// independent of later changes to the administrator used for job attribution.
pub(super) async fn automatic(db: &mut PgConnection, job: &Value) -> Result<bool> {
    if !matches!(
        job["kind"].as_str(),
        Some("official.backup" | "official.backup.prune")
    ) {
        return Ok(false);
    }
    let valid: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM backups WHERE id=$1 AND server_id=$2 AND kind='official' AND ((job_id=$3 AND scheduled_for IS NOT NULL AND $4='official.backup') OR (prune_job_id=$3 AND state='pruning' AND NOT pinned AND $4='official.backup.prune')))")
        .bind(uuid(&job["payload"],"backup_id")?).bind(uuid(job,"server_id")?).bind(uuid(job,"id")?).bind(job["kind"].as_str().unwrap()).fetch_one(db).await?;
    Ok(valid)
}

async fn prune_record(
    db: &mut PgConnection,
    id: Uuid,
    service: Uuid,
    lease: Uuid,
) -> Result<Value> {
    let job: Value = sqlx::query_scalar("SELECT to_jsonb(j) FROM jobs j WHERE id=$1 AND kind='official.backup.prune' AND state='leased' AND lease_owner=$2 AND lease_token=$3 AND lease_until>now() AND host_authorized_at IS NOT NULL FOR UPDATE")
        .bind(id).bind(service).bind(lease).fetch_optional(&mut *db).await?.ok_or_else(Error::forbidden)?;
    sqlx::query_scalar("SELECT to_jsonb(b) FROM backups b WHERE id=$1 AND server_id=$2 AND kind='official' AND state='pruning' AND prune_job_id=$3 AND NOT pinned FOR UPDATE")
        .bind(uuid(&job["payload"],"backup_id")?).bind(uuid(&job,"server_id")?).bind(id).fetch_optional(db).await?.ok_or_else(Error::forbidden)
}

pub async fn prune(
    State(app): State<App>,
    service: Service,
    Path(id): Path<Uuid>,
    Json(request): Json<Lease>,
) -> Result<Json<Value>> {
    service.require("host")?;
    let mut tx = app.db.begin().await?;
    let record = prune_record(&mut tx, id, service.id, request.lease_token).await?;
    let backup = uuid(&record, "id")?;
    let result = json!({"backup_id":backup,"database_deleted":true});
    if !record["database_pruned_at"].is_null() {
        return Ok(Json(result));
    }
    tx.commit().await?;
    let root = backup::directory(&app)?;
    let path = root.join(format!("{backup}.dump"));
    if record["database_prune_started_at"].is_null() {
        let manifest = &record["manifest"]["database"];
        backup::valid_archive(manifest)?;
        let metadata = std::fs::symlink_metadata(&path).map_err(Error::internal)?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || Some(metadata.len()) != manifest["bytes"].as_u64()
        {
            return Err(Error::conflict(
                "The database file to prune could not be verified.",
            ));
        }
        let mut file = tokio::fs::File::open(&path)
            .await
            .map_err(Error::internal)?;
        let mut hash = Sha256::new();
        let mut buffer = vec![0u8; 1024 * 1024];
        loop {
            let n = file.read(&mut buffer).await.map_err(Error::internal)?;
            if n == 0 {
                break;
            }
            hash.update(&buffer[..n]);
        }
        if Some(hex::encode(hash.finalize()).as_str()) != manifest["sha256"].as_str() {
            return Err(Error::conflict(
                "The database hash does not match the saved pruning record.",
            ));
        }
    }
    // Persist intent before unlink. Lost replies or a process crash resume only
    // this exact backup; no filename supplied by a caller is ever used.
    let mut tx = app.db.begin().await?;
    prune_record(&mut tx, id, service.id, request.lease_token).await?;
    sqlx::query("UPDATE backups SET database_prune_started_at=coalesce(database_prune_started_at,now()) WHERE id=$1")
        .bind(backup).execute(&mut *tx).await?;
    tx.commit().await?;
    for path in [path, root.join(format!("{backup}.lock"))] {
        match std::fs::symlink_metadata(&path) {
            Ok(meta) if meta.is_file() && !meta.file_type().is_symlink() => {
                std::fs::remove_file(path).map_err(Error::internal)?
            }
            Ok(_) => {
                return Err(Error::conflict("The pruning target is not a regular file."));
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(Error::internal(e)),
        }
    }
    File::open(root)
        .and_then(|f| f.sync_all())
        .map_err(Error::internal)?;
    let mut tx = app.db.begin().await?;
    prune_record(&mut tx, id, service.id, request.lease_token).await?;
    sqlx::query(
        "UPDATE backups SET database_pruned_at=coalesce(database_pruned_at,now()) WHERE id=$1",
    )
    .bind(backup)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(result))
}
