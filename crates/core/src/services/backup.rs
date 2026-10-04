use super::{Service, uuid};
use crate::{
    App,
    error::{Error, Result},
};
use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::header,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{
    PgConnection,
    postgres::{PgConnectOptions, PgSslMode},
};
use std::{
    fs::{File, OpenOptions},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::PathBuf,
    process::Stdio,
    str::FromStr,
    time::Duration,
};
use tokio::io::AsyncReadExt;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Request {
    lease_token: Uuid,
    action: String,
    world_manifest: Option<Value>,
}

/// The game process must already be stopped. Durable ownership prevents a late
/// worker from releasing the barrier belonging to another backup.
pub async fn control(
    State(app): State<App>,
    service: Service,
    Path(id): Path<Uuid>,
    Json(request): Json<Request>,
) -> Result<Json<Value>> {
    service.require("host")?;
    let mut tx = app.db.begin().await?;
    let job:Value=sqlx::query_scalar("SELECT to_jsonb(j) FROM jobs j WHERE id=$1 AND kind='official.backup' AND state='leased' AND lease_owner=$2 AND lease_token=$3 AND lease_until>now() FOR UPDATE").bind(id).bind(service.id).bind(request.lease_token).fetch_optional(&mut *tx).await?.ok_or_else(Error::forbidden)?;
    let backup = uuid(&job["payload"], "backup_id")?;
    let owner: Value = sqlx::query_scalar(
        "SELECT value FROM settings WHERE key='official_backup_owner' FOR UPDATE",
    )
    .fetch_one(&mut *tx)
    .await?;
    if !owner.is_null() && owner != json!(id) {
        return Err(Error::conflict(
            "text.another_official_backup_is_being_recovered",
        ));
    }
    let existing:Option<Value>=sqlx::query_scalar("SELECT to_jsonb(b) FROM official_backup_steps b WHERE backup_id=$1 AND job_id=$2 FOR UPDATE").bind(backup).bind(id).fetch_optional(&mut *tx).await?;
    if request.action == "freeze" {
        if let Some(step) = existing {
            tx.commit().await?;
            return Ok(Json(step));
        }
        let stopped:bool=sqlx::query_scalar("SELECT kind='official' AND maintenance AND maintenance_job_id=$2 AND observed='stopped' AND players=0 AND last_observed_at>now()-interval '45 seconds' FROM servers WHERE id=$1 FOR UPDATE").bind(uuid(&job,"server_id")?).bind(id).fetch_one(&mut *tx).await?;
        if !stopped {
            return Err(Error::conflict(
                "text.confirm_the_official_smp_has_stopped_and_saved_before_f_d833796790",
            ));
        }
        // Wait for existing official transactions holding a shared barrier lock.
        sqlx::query("UPDATE settings SET value='true' WHERE key='official_mutations_paused'")
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE settings SET value=$1 WHERE key='official_backup_owner'")
            .bind(json!(id))
            .execute(&mut *tx)
            .await?;
        let step:Value=sqlx::query_scalar("INSERT INTO official_backup_steps(backup_id,job_id,phase) VALUES($1,$2,'frozen') RETURNING to_jsonb(official_backup_steps)").bind(backup).bind(id).fetch_one(&mut *tx).await?;
        sqlx::query("UPDATE backups SET state='freezing',error=NULL WHERE id=$1")
            .bind(backup)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        return Ok(Json(step));
    }
    let step = existing.ok_or_else(|| Error::conflict("text.freeze_official_state_first"))?;
    if request.action == "status" {
        tx.commit().await?;
        return Ok(Json(step));
    }
    if request.action == "release" {
        if step["phase"] == "released" {
            tx.commit().await?;
            return Ok(Json(step));
        }
        if step["phase"] != "dumped" || owner != json!(id) {
            return Err(Error::conflict(
                "text.access_cannot_resume_until_the_database_backup_is_confirmed",
            ));
        }
        let world = request
            .world_manifest
            .ok_or_else(|| Error::invalid("text.the_world_save_record_is_missing"))?;
        valid_archive(&world)?;
        if world["server_id"] != job["server_id"] {
            return Err(Error::invalid("text.the_world_save_source_does_not_match"));
        }
        sqlx::query("UPDATE official_backup_steps SET phase='released',world_manifest=$2,released_at=now() WHERE backup_id=$1").bind(backup).bind(world).execute(&mut *tx).await?;
        release(&mut tx, id).await?;
        tx.commit().await?;
        return Ok(Json(json!({"phase":"released"})));
    }
    if request.action != "dump" {
        return Err(Error::invalid("text.the_backup_operation_is_invalid"));
    }
    if step["phase"] == "dumped" || step["phase"] == "released" {
        tx.commit().await?;
        return Ok(Json(step));
    }
    if owner != json!(id) {
        return Err(Error::conflict("text.you_do_not_own_the_save_barrier"));
    }
    let directory = directory(&app)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(directory.join(format!("{backup}.lock")))
        .map_err(Error::internal)?;
    if file.try_lock().is_err() {
        tx.commit().await?;
        return Ok(Json(step));
    }
    sqlx::query("UPDATE official_backup_steps SET phase='dumping',error=NULL WHERE backup_id=$1")
        .bind(backup)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE backups SET state='saving',error=NULL WHERE id=$1")
        .bind(backup)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    tokio::spawn(async move {
        let _lock = file;
        match dump(&app, backup).await {
            Ok(manifest) => {
                // A reissued lease cannot accept a late dump result from its predecessor.
                let result=sqlx::query("WITH saved AS (UPDATE official_backup_steps b SET phase='dumped',database_manifest=$2,error=NULL FROM jobs j WHERE b.backup_id=$1 AND b.job_id=j.id AND b.phase='dumping' AND j.state='leased' AND j.lease_owner=$3 AND j.lease_token=$4 AND j.lease_until>now() AND (SELECT value FROM settings WHERE key='official_backup_owner')=to_jsonb(j.id) RETURNING b.backup_id) UPDATE backups SET state='verifying',error=NULL WHERE id IN (SELECT backup_id FROM saved)").bind(backup).bind(manifest).bind(service.id).bind(request.lease_token).execute(&app.db).await;
                if let Err(e) = result {
                    tracing::error!(%backup,error=%e,"Database dump awaits reconciliation");
                }
            }
            Err(error) => {
                tracing::error!(%backup,error=%error,"Official dump failed; barrier remains held");
                let _=sqlx::query("UPDATE official_backup_steps SET phase='frozen',error=$2 WHERE backup_id=$1 AND phase='dumping'").bind(backup).bind(error.to_string()).execute(&app.db).await;
            }
        }
    });
    Ok(Json(json!({"phase":"dumping"})))
}

pub(super) async fn release(db: &mut PgConnection, job: Uuid) -> Result<()> {
    let owner: Value = sqlx::query_scalar(
        "SELECT value FROM settings WHERE key='official_backup_owner' FOR UPDATE",
    )
    .fetch_one(&mut *db)
    .await?;
    if owner != json!(job) {
        return Err(Error::conflict("text.save_barrier_ownership_has_changed"));
    }
    sqlx::query("UPDATE settings SET value='false' WHERE key='official_mutations_paused'")
        .execute(&mut *db)
        .await?;
    sqlx::query("UPDATE settings SET value='null' WHERE key='official_backup_owner'")
        .execute(db)
        .await?;
    Ok(())
}

pub(super) fn valid_archive(value: &Value) -> Result<()> {
    let sha = value["sha256"].as_str().unwrap_or("");
    if sha.len() != 64
        || !sha.bytes().all(|b| b.is_ascii_hexdigit())
        || value["bytes"].as_u64().unwrap_or(0) == 0
    {
        return Err(Error::invalid(
            "text.the_saved_file_s_size_or_sha256_is_invalid",
        ));
    }
    Ok(())
}

pub(super) fn directory(app: &App) -> Result<PathBuf> {
    let p = app.config.storage.join("official-backups");
    std::fs::create_dir_all(&p).map_err(Error::internal)?;
    if std::fs::symlink_metadata(&p)
        .map_err(Error::internal)?
        .file_type()
        .is_symlink()
    {
        return Err(Error::internal("Backup directory is a symlink"));
    }
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o700))
        .map_err(Error::internal)?;
    Ok(p)
}

async fn dump(app: &App, id: Uuid) -> anyhow::Result<Value> {
    let root = directory(app)?;
    let temporary = root.join(format!("{id}.{}.partial", Uuid::new_v4()));
    let _temporary = IncompleteDump(temporary.clone());
    let output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)?;
    let options = PgConnectOptions::from_str(&app.config.database_url)?;
    let url = reqwest::Url::parse(&app.config.database_url)?;
    let password = url
        .password()
        .map(|p| {
            percent_encoding::percent_decode_str(p)
                .decode_utf8()
                .map(|s| s.into_owned())
        })
        .transpose()?;
    let mut command = tokio::process::Command::new(&app.config.pg_dump);
    command
        .args([
            "--format=custom",
            "--no-owner",
            "--no-privileges",
            "--no-password",
        ])
        .env(
            "PGHOST",
            options
                .get_socket()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| options.get_host().into()),
        )
        .env("PGPORT", options.get_port().to_string())
        .env("PGUSER", options.get_username())
        .env(
            "PGDATABASE",
            options.get_database().unwrap_or(options.get_username()),
        )
        .env(
            "PGSSLMODE",
            match options.get_ssl_mode() {
                PgSslMode::Disable => "disable",
                PgSslMode::Allow => "allow",
                PgSslMode::Prefer => "prefer",
                PgSslMode::Require => "require",
                PgSslMode::VerifyCa => "verify-ca",
                PgSslMode::VerifyFull => "verify-full",
            },
        )
        .env_remove("PGSERVICE")
        .env_remove("PGSERVICEFILE")
        .env_remove("PGOPTIONS")
        .stdin(Stdio::null())
        .stdout(Stdio::from(output.try_clone()?))
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    if let Some(password) = password {
        command.env("PGPASSWORD", password);
    }
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "sslrootcert" => {
                command.env("PGSSLROOTCERT", value.as_ref());
            }
            "sslcert" => {
                command.env("PGSSLCERT", value.as_ref());
            }
            "sslkey" => {
                command.env("PGSSLKEY", value.as_ref());
            }
            "password" => {
                command.env("PGPASSWORD", value.as_ref());
            }
            _ => {}
        }
    }
    // Command::output would replace the configured stdout file with an in-memory pipe.
    let mut child = command.spawn()?;
    let stderr = child.stderr.take().unwrap();
    let (status, errors) = tokio::time::timeout(Duration::from_secs(3600), async {
        tokio::try_join!(
            async { Ok::<_, anyhow::Error>(child.wait().await?) },
            async {
                let mut errors = Vec::new();
                stderr.take(65537).read_to_end(&mut errors).await?;
                anyhow::ensure!(
                    errors.len() <= 65536,
                    "pg_dump diagnostics exceeded the limit"
                );
                Ok::<_, anyhow::Error>(errors)
            }
        )
    })
    .await??;
    // Do not include connection strings or subprocess environment in logs.
    anyhow::ensure!(
        status.success(),
        "pg_dump failed (status {}); database credentials remain on Core",
        status
    );
    anyhow::ensure!(
        errors.is_empty(),
        "pg_dump reported warnings; inspect the database before accepting this backup"
    );
    output.sync_all()?;
    let restore = app.config.pg_dump.with_file_name("pg_restore");
    let status = tokio::time::timeout(
        Duration::from_secs(60),
        tokio::process::Command::new(restore)
            .arg("--list")
            .arg(&temporary)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .status(),
    )
    .await??;
    anyhow::ensure!(
        status.success(),
        "pg_restore cannot read the complete dump catalog"
    );
    let mut file = tokio::fs::File::open(&temporary).await?;
    let mut hash = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = vec![0; 1048576];
    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
        bytes += n as u64;
    }
    let target = root.join(format!("{id}.dump"));
    std::fs::rename(&temporary, &target)?;
    File::open(&root)?.sync_all()?;
    let database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&app.db)
        .await?;
    Ok(
        json!({"sha256":hex::encode(hash.finalize()),"bytes":bytes,"format":"postgres-custom","database":database,"verification":"sha256-and-pg-restore-list","restore_tested":false}),
    )
}

struct IncompleteDump(PathBuf);
impl Drop for IncompleteDump {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

pub async fn download(
    State(app): State<App>,
    service: Service,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    service.require("host")?;
    let manifest:Value=sqlx::query_scalar("SELECT s.database_manifest FROM official_backup_steps s JOIN backups b ON b.id=s.backup_id WHERE s.backup_id=$1 AND s.phase IN ('dumped','released') AND b.state NOT IN ('pruning','pruned')").bind(id).fetch_optional(&app.db).await?.ok_or_else(Error::missing)?;
    let file = tokio::fs::File::open(directory(&app)?.join(format!("{id}.dump")))
        .await
        .map_err(Error::internal)?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/octet-stream".to_string()),
            (header::CONTENT_LENGTH, manifest["bytes"].to_string()),
            (header::CACHE_CONTROL, "no-store".to_string()),
        ],
        Body::from_stream(tokio_util::io::ReaderStream::new(file)),
    )
        .into_response())
}
