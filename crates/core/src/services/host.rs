use super::{Service, uuid};
use crate::{
    App,
    error::{Error, Result},
};
use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Lease {
    pub lease_token: Uuid,
}
pub async fn context(
    State(app): State<App>,
    service: Service,
    Path(id): Path<Uuid>,
    Json(request): Json<Lease>,
) -> Result<Json<Value>> {
    service.require("host")?;
    let mut tx = app.db.begin().await?;
    let job:Value=sqlx::query_scalar("SELECT to_jsonb(j) FROM jobs j WHERE id=$1 AND lease_owner=$2 AND lease_token=$3 AND state='leased' AND lease_until>now() FOR UPDATE")
        .bind(id).bind(service.id).bind(request.lease_token).fetch_optional(&mut *tx).await?.ok_or_else(Error::forbidden)?;
    let server = uuid(&job, "server_id")?;
    let data: Value =
        sqlx::query_scalar("SELECT to_jsonb(s) FROM servers s WHERE id=$1 FOR UPDATE")
            .bind(server)
            .fetch_one(&mut *tx)
            .await?;
    // Once an operation has crossed this durable authorization boundary, recovery
    // must finish even if its submitter is subsequently removed or banned.
    if job["host_authorized_at"].is_null() || job["kind"] == "server.logs" {
        if let Err(e) = authorize(&mut tx, &job, &data).await {
            if e.status.is_server_error() {
                return Err(e);
            }
            return Ok(Json(json!({"rejected":e.message,"effect":"none"})));
        }
    }
    if !matches!(
        job["kind"].as_str(),
        Some("server.logs" | "official.backup.prune")
    ) {
        if !data["maintenance_job_id"].is_null() && data["maintenance_job_id"] != job["id"] {
            return Err(Error::conflict(
                "先に開始したサーバー処理の回復を待っています。",
            ));
        }
        sqlx::query("UPDATE servers SET maintenance=true,maintenance_job_id=$2 WHERE id=$1")
            .bind(server)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    if job["kind"] != "server.logs" {
        sqlx::query(
            "UPDATE jobs SET host_authorized_at=coalesce(host_authorized_at,now()) WHERE id=$1",
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;
    }
    let mut result = json!({"server":data,"job":job});
    if let Some(id) = job["payload"].get("artifact_id") {
        let artifact = Uuid::parse_str(id.as_str().unwrap_or(""))
            .map_err(|_| Error::invalid("ファイルIDが不正です。"))?;
        result["artifact"] = sqlx::query_scalar::<_, Value>(
            "SELECT to_jsonb(a) FROM artifacts a WHERE id=$1 AND server_id=$2",
        )
        .bind(artifact)
        .bind(server)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(Error::missing)?;
    }
    if let Some(id) = job["payload"].get("backup_id") {
        let backup = Uuid::parse_str(id.as_str().unwrap_or(""))
            .map_err(|_| Error::invalid("バックアップIDが不正です。"))?;
        result["backup"] = sqlx::query_scalar::<_, Value>(
            "SELECT to_jsonb(b) FROM backups b WHERE id=$1 AND server_id=$2",
        )
        .bind(backup)
        .bind(server)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(Error::missing)?;
    }
    tx.commit().await?;
    Ok(Json(result))
}

async fn authorize(db: &mut PgConnection, job: &Value, server: &Value) -> Result<()> {
    if super::backup_policy::automatic(db, job).await? {
        return Ok(());
    }
    let actor = uuid(job, "actor")?;
    let id = uuid(server, "id")?;
    let active:bool=sqlx::query_scalar("SELECT merged_into IS NULL AND (banned_until IS NULL OR banned_until<now()) FROM accounts WHERE id=$1").bind(actor).fetch_one(&mut *db).await?;
    if !active && !(job["kind"] == "server.stop" && job["payload"]["idle"] == true) {
        return Err(Error::forbidden());
    }
    match job["kind"].as_str().unwrap_or("") {
        // A joining player may wake a public server without being an operator.
        "server.start" => {
            crate::hosting::can_remain(db, actor, id).await?;
            if server["desired"] != "running" {
                return Err(Error::conflict("後から受け付けた停止要求を優先しました。"));
            }
        }
        "server.logs" | "server.stop" => {
            crate::hosting::server_permission(db, actor, id, false).await?
        }
        "server.create" | "server.console" | "server.install" | "server.backup"
        | "server.restore" | "official.backup" => {
            crate::hosting::server_permission(db, actor, id, true).await?;
        }
        _ => return Err(Error::invalid("実行できないホスト操作です。")),
    }
    if matches!(
        job["kind"].as_str(),
        Some("server.install" | "server.restore")
    ) && (server["kind"] != "custom"
        || server["desired"] != "stopped"
        || server["observed"] != "stopped")
    {
        return Err(Error::conflict(
            "ファイル反映・復元には個人サーバーの停止が必要です。",
        ));
    }
    if job["kind"] == "server.console"
        && (server["desired"] != "running" || server["observed"] != "running")
    {
        return Err(Error::conflict(
            "コンソールを送信する前にサーバーを起動してください。",
        ));
    }
    Ok(())
}
