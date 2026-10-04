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
    let mut job:Value=sqlx::query_scalar("SELECT to_jsonb(j) FROM jobs j WHERE id=$1 AND lease_owner=$2 AND lease_token=$3 AND state='leased' AND lease_until>now() FOR UPDATE")
        .bind(id).bind(service.id).bind(request.lease_token).fetch_optional(&mut *tx).await?.ok_or_else(Error::forbidden)?;
    let server = uuid(&job, "server_id")?;
    let mut data: Value =
        sqlx::query_scalar("SELECT to_jsonb(s) FROM servers s WHERE id=$1 FOR UPDATE")
            .bind(server)
            .fetch_one(&mut *tx)
            .await?;
    if crate::server_tools::passive(job["kind"].as_str().unwrap_or("")) {
        let restoring: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM jobs WHERE server_id=$1 AND kind='server.restore' AND state IN ('queued','leased','waiting') AND (id=(SELECT maintenance_job_id FROM servers WHERE id=$1) OR state='leased'))")
            .bind(server).fetch_one(&mut *tx).await?;
        if restoring {
            return Ok(Json(
                json!({"rejected":crate::system_message::SystemMessage::new("text.reading_is_unavailable_while_this_server_is_being_restored"),"effect":"none"}),
            ));
        }
    }
    let inspecting = !data["inspection"].is_null();
    let inspection_valid = crate::server_tools::inspection_valid(&mut tx, &data).await?;
    data["inspection_valid"] = json!(inspection_valid);
    if job["kind"] == "server.inspection" {
        if data["inspection"]["id"] != job["payload"]["inspection"]["id"] {
            return Ok(Json(
                json!({"rejected":crate::system_message::SystemMessage::new("text.the_inspection_session_changed"),"effect":"none"}),
            ));
        }
    } else if inspecting {
        if !matches!(
            job["kind"].as_str(),
            Some(
                "server.logs"
                    | "server.files"
                    | "server.file.read"
                    | "server.file.write"
                    | "server.file.delete"
                    | "server.directory.create"
                    | "server.operator"
                    | "server.install"
            )
        ) {
            return Ok(Json(
                json!({"rejected":crate::system_message::SystemMessage::new("text.close_file_inspection_before_this_operation"),"effect":"none"}),
            ));
        }
        if !inspection_valid && job["host_authorized_at"].is_null() {
            return Ok(Json(
                json!({"rejected":crate::system_message::SystemMessage::new("text.the_file_inspection_window_expired_or_its_authorization_65e8e28026"),"effect":"none"}),
            ));
        }
    }
    // An OP authorization/identity change after prepare cannot prove no effect:
    // a guest may already have renamed ops.json before its acknowledgement.
    let rejection_effect =
        if job["kind"] == "server.operator" && !job["host_authorized_at"].is_null() {
            "uncertain"
        } else {
            "none"
        };
    // Once an operation has crossed this durable authorization boundary, recovery
    // must finish even if its submitter is subsequently removed or banned.
    if job["host_authorized_at"].is_null()
        || crate::server_tools::passive(job["kind"].as_str().unwrap_or(""))
        || job["kind"] == "server.operator"
        || inspecting
    {
        if job["kind"] != "server.inspection" {
            if let Err(e) = authorize(&mut tx, &job, &data).await {
                if e.status.is_server_error() {
                    return Err(e);
                }
                if !job["host_authorized_at"].is_null()
                    && matches!(
                        job["kind"].as_str(),
                        Some(
                            "server.install"
                                | "server.file.write"
                                | "server.file.delete"
                                | "server.directory.create"
                        )
                    )
                {
                    data["reconcile_only"] = json!(true);
                } else {
                    return Ok(Json(
                        json!({"rejected":e.message,"effect":rejection_effect}),
                    ));
                }
            }
        }
    }
    if inspecting && !inspection_valid {
        data["reconcile_only"] = json!(true);
    }
    if !matches!(
        job["kind"].as_str(),
        Some("server.logs" | "server.files" | "server.file.read" | "official.backup.prune")
    ) {
        if !data["maintenance_job_id"].is_null() && data["maintenance_job_id"] != job["id"] {
            return Err(Error::conflict(
                "text.waiting_for_an_earlier_server_operation_to_recover",
            ));
        }
        sqlx::query("UPDATE servers SET maintenance=true,maintenance_job_id=$2 WHERE id=$1")
            .bind(server)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    if !crate::server_tools::passive(job["kind"].as_str().unwrap_or("")) {
        job["host_authorized_at"] = sqlx::query_scalar::<_, Value>(
            "UPDATE jobs SET host_authorized_at=coalesce(host_authorized_at,now()) WHERE id=$1 RETURNING to_jsonb(host_authorized_at)",
        ).bind(id).fetch_one(&mut *tx).await?;
    }
    if job["kind"] == "server.operator" {
        let member = uuid(&job["payload"], "member")?;
        let native = match crate::server_tools::identity(
            &mut tx,
            server,
            member,
            job["payload"]["operator"] == true,
        )
        .await
        {
            Ok(v) => v,
            Err(e) if !e.status.is_server_error() => {
                return Ok(Json(
                    json!({"rejected":e.message,"effect":rejection_effect}),
                ));
            }
            Err(e) => return Err(e),
        };
        if native != job["payload"]["identity"] {
            return Ok(Json(
                json!({"rejected":crate::system_message::SystemMessage::new("text.the_verified_minecraft_identity_changed_submit_a_new_op_request"),"effect":rejection_effect}),
            ));
        }
    }
    let mut result = json!({"server":data,"job":job});
    if let Some(id) = job["payload"].get("artifact_id") {
        let artifact = Uuid::parse_str(id.as_str().unwrap_or(""))
            .map_err(|_| Error::invalid("text.the_file_id_is_invalid"))?;
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
            .map_err(|_| Error::invalid("text.the_backup_id_is_invalid"))?;
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
                return Err(Error::conflict("text.a_newer_stop_request_took_precedence"));
            }
        }
        "server.logs" | "server.stop" => {
            crate::hosting::server_permission(db, actor, id, false).await?
        }
        "server.create"
        | "server.console"
        | "server.install"
        | "server.backup"
        | "server.restore"
        | "official.backup"
        | "server.files"
        | "server.file.read"
        | "server.file.write"
        | "server.file.delete"
        | "server.directory.create"
        | "server.operator" => {
            crate::hosting::server_permission(db, actor, id, true).await?;
        }
        _ => return Err(Error::invalid("text.this_host_operation_is_not_supported")),
    }
    if matches!(
        job["kind"].as_str(),
        Some(
            "server.install"
                | "server.restore"
                | "server.file.write"
                | "server.file.delete"
                | "server.directory.create"
                | "server.operator"
        )
    ) && (server["kind"] != "custom"
        || server["desired"] != "stopped"
        || server["observed"] != "stopped")
    {
        return Err(Error::conflict(
            "text.stop_the_personal_server_before_applying_files_or_resto_101e19587d",
        ));
    }
    if job["kind"] == "server.operator" && server["software"] != "paper" {
        return Err(Error::conflict(
            "text.minecraft_op_is_supported_only_on_stopped_custom_paper_servers",
        ));
    }
    if job["kind"] == "server.console"
        && (server["desired"] != "running" || server["observed"] != "running")
    {
        return Err(Error::conflict(
            "text.start_the_server_before_sending_a_console_command",
        ));
    }
    Ok(())
}
