use crate::{
    App,
    auth::{Actor, audit},
    commands::{Command, job, label},
    error::{Error, Result},
};
use axum::{
    Json,
    extract::{Multipart, Path, State},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{PgConnection, Row};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub async fn server_permission(
    db: &mut PgConnection,
    actor: Uuid,
    id: Uuid,
    administer: bool,
) -> Result<()> {
    let allowed:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM servers s WHERE s.id=$1 AND (s.owner=$2 OR EXISTS(SELECT 1 FROM accounts a WHERE a.id=$2 AND a.administrator) OR EXISTS(SELECT 1 FROM server_members m WHERE m.server_id=s.id AND m.account_id=$2 AND (m.role='administrator' OR (NOT $3 AND m.role='operator'))) OR EXISTS(SELECT 1 FROM communities c JOIN community_members m ON m.community_id=c.id WHERE c.id=s.community_id AND m.account_id=$2 AND m.administrator)))")
        .bind(id).bind(actor).bind(administer).fetch_one(db).await?;
    if !allowed {
        return Err(Error::forbidden());
    }
    Ok(())
}
pub async fn can_join(db: &mut PgConnection, actor: Uuid, id: Uuid) -> Result<()> {
    can_remain(db, actor, id).await?;
    let maintenance: bool = sqlx::query_scalar("SELECT maintenance FROM servers WHERE id=$1")
        .bind(id)
        .fetch_one(db)
        .await?;
    if maintenance {
        return Err(Error::unavailable(
            "Saving, stopping, or maintenance is in progress. Please wait in the lobby.",
        ));
    }
    Ok(())
}
pub async fn can_remain(db: &mut PgConnection, actor: Uuid, id: Uuid) -> Result<()> {
    let allowed:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM servers s WHERE s.id=$1 AND (s.visibility='public' OR s.owner=$2 OR EXISTS(SELECT 1 FROM accounts WHERE id=$2 AND administrator) OR EXISTS(SELECT 1 FROM server_members WHERE server_id=s.id AND account_id=$2) OR EXISTS(SELECT 1 FROM community_members WHERE community_id=s.community_id AND account_id=$2)))")
        .bind(id).bind(actor).fetch_one(db).await?;
    if !allowed {
        return Err(Error::forbidden());
    }
    Ok(())
}
pub async fn wake(db: &mut PgConnection, actor: Uuid, id: Uuid) -> Result<Value> {
    wake_request(db, actor, id, false).await
}
pub(crate) async fn wake_for_join(db: &mut PgConnection, actor: Uuid, id: Uuid) -> Result<Value> {
    wake_request(db, actor, id, true).await
}
async fn wake_request(
    db: &mut PgConnection,
    actor: Uuid,
    id: Uuid,
    automatic: bool,
) -> Result<Value> {
    let row = sqlx::query("SELECT * FROM servers WHERE id=$1 FOR UPDATE")
        .bind(id)
        .fetch_optional(&mut *db)
        .await?
        .ok_or_else(Error::missing)?;
    let inspection: Option<Value> = row.get("inspection");
    if inspection.is_some() {
        server_permission(db, actor, id, true).await?;
        let busy: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM jobs WHERE server_id=$1 AND worker='host' AND state='leased')").bind(id).fetch_one(&mut *db).await?;
        if busy || row.get::<Option<Uuid>, _>("maintenance_job_id").is_some() {
            return Err(Error::conflict(
                "Wait for the current file operation before starting Minecraft.",
            ));
        }
        // Transfer guest ownership under the server row lock; stale cleanup cannot stop the game.
        sqlx::query("UPDATE servers SET inspection=NULL,maintenance=false WHERE id=$1")
            .bind(id)
            .execute(&mut *db)
            .await?;
        sqlx::query("UPDATE jobs SET state='cancelled',result='{\"effect\":\"superseded\"}'::jsonb,progress='{\"phase\":\"cancelled\"}'::jsonb,updated_at=now() WHERE server_id=$1 AND kind='server.inspection' AND state IN ('queued','waiting')").bind(id).execute(&mut *db).await?;
    }
    if row.get::<bool, _>("maintenance") && inspection.is_none() {
        return Err(Error::unavailable("This server is under maintenance."));
    }
    let observed: String = row.get("observed");
    if observed == "running"
        && row
            .get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_observed_at")
            .is_some_and(|t| t > chrono::Utc::now() - chrono::Duration::seconds(45))
    {
        return Ok(json!({"server_id":id,"state":"running"}));
    }
    if let Some(job_id)=sqlx::query_scalar::<_,Uuid>("SELECT id FROM jobs WHERE server_id=$1 AND kind IN ('server.create','server.start','server.stop','server.restore') AND state IN ('queued','leased','waiting')").bind(id).fetch_optional(&mut *db).await? {
        return Ok(json!({"job_id":job_id,"state":"waiting"}));
    }
    reserve_capacity(db, id).await?;
    if observed == "unprovisioned" {
        return Err(Error::conflict("Server creation has not finished."));
    }
    sqlx::query("UPDATE servers SET desired='running',error=NULL WHERE id=$1")
        .bind(id)
        .execute(&mut *db)
        .await?;
    job(
        db,
        actor,
        Some(id),
        "host",
        "server.start",
        json!({"server_id":id,"automatic":automatic}),
    )
    .await
}
/// Running games and admitted file guests share the same owner resource allowance.
pub(crate) async fn reserve_capacity(db: &mut PgConnection, id: Uuid) -> Result<()> {
    let row = sqlx::query("SELECT * FROM servers WHERE id=$1")
        .bind(id)
        .fetch_one(&mut *db)
        .await?;
    if row.get::<String, _>("kind") != "custom" {
        return Ok(());
    }
    let owner: Uuid = row.get("owner");
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("quota:{owner}"))
        .execute(&mut *db)
        .await?;
    let rank = sqlx::query(
        "SELECT r.* FROM trust_ranks r JOIN accounts a ON a.trust_rank=r.id WHERE a.id=$1",
    )
    .bind(owner)
    .fetch_one(&mut *db)
    .await?;
    let usage = sqlx::query("SELECT count(*) AS count,coalesce(sum(memory_mib),0)::bigint AS memory,coalesce(sum(cpu_millis),0)::bigint AS cpu FROM servers WHERE owner=$1 AND id<>$2 AND (desired='running' OR inspection IS NOT NULL)").bind(owner).bind(id).fetch_one(&mut *db).await?;
    if usage.get::<i64, _>("count") >= rank.get::<i32, _>("concurrent_servers") as i64
        || usage.get::<i64, _>("memory") + row.get::<i32, _>("memory_mib") as i64
            > rank.get::<i32, _>("memory_mib") as i64
        || usage.get::<i64, _>("cpu") + row.get::<i32, _>("cpu_millis") as i64
            > rank.get::<i32, _>("cpu_millis") as i64
    {
        return Err(Error::conflict(
            "This exceeds your tier’s concurrent server, memory, or CPU allowance. Stop another server or close its files first.",
        ));
    }
    Ok(())
}
fn visibility(s: &str) -> Result<()> {
    if !matches!(s, "public" | "invite" | "private") {
        return Err(Error::invalid("The visibility setting is invalid."));
    }
    Ok(())
}
pub fn safe_path(path: &str) -> Result<()> {
    if path.len() > 240
        || path.is_empty()
        || path.contains('\\')
        || path.starts_with('/')
        || path.chars().any(char::is_control)
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == ".." || part.starts_with('.'))
    {
        return Err(Error::invalid("The file destination is invalid."));
    }
    if crate::server_tools::protected(path) {
        return Err(Error::invalid("System-managed files cannot be changed."));
    }
    Ok(())
}

pub async fn command(db: &mut PgConnection, actor: &Actor, command: &Command) -> Result<Value> {
    use Command::*;
    let me = actor.id;
    match command {
        ServerCreate {
            name,
            software,
            version,
            memory_mib,
            cpu_millis,
            storage_mib,
            visibility: v,
            community,
        } => {
            visibility(v)?;
            let name = label(name, 64)?;
            let version = label(version, 32)?;
            if !matches!(
                software.as_str(),
                "paper" | "fabric" | "forge" | "neoforge" | "custom"
            ) {
                return Err(Error::invalid("The server software is invalid."));
            }
            if *memory_mib < 512 || *cpu_millis < 100 || *storage_mib < 1024 {
                return Err(Error::invalid("The resource allocation is too small."));
            }
            if *cpu_millis % 1000 != 0 {
                return Err(Error::invalid(
                    "Specify whole CPU cores for virtual machines.",
                ));
            }
            if let Some(community) = community {
                let member:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM community_members WHERE community_id=$1 AND account_id=$2 AND administrator)").bind(community).bind(me).fetch_one(&mut *db).await?;
                if !member {
                    return Err(Error::forbidden());
                }
            }
            let rank = sqlx::query(
                "SELECT r.* FROM trust_ranks r JOIN accounts a ON a.trust_rank=r.id WHERE a.id=$1",
            )
            .bind(me)
            .fetch_one(&mut *db)
            .await?;
            let usage=sqlx::query("SELECT count(*) AS count,coalesce(sum(storage_mib),0)::bigint AS storage FROM servers WHERE owner=$1").bind(me).fetch_one(&mut *db).await?;
            if usage.get::<i64, _>("count") >= rank.get::<i32, _>("server_count") as i64
                || usage.get::<i64, _>("storage") + storage_mib > rank.get::<i64, _>("storage_mib")
                || memory_mib > &rank.get::<i32, _>("memory_mib")
                || cpu_millis > &rank.get::<i32, _>("cpu_millis")
            {
                return Err(Error::conflict(
                    "This exceeds your server creation allowance. Ask an administrator to approve a higher tier.",
                ));
            }
            let id = Uuid::new_v4();
            // Capabilities are populated only after the host and proxy have verified the installed runtime.
            sqlx::query("INSERT INTO servers(id,owner,community_id,name,kind,visibility,version,software,memory_mib,cpu_millis,storage_mib) VALUES($1,$2,$3,$4,'custom',$5,$6,$7,$8,$9,$10)")
                .bind(id).bind(me).bind(community).bind(name).bind(v).bind(version).bind(software).bind(memory_mib).bind(cpu_millis).bind(storage_mib).execute(&mut *db).await?;
            let result = job(
                db,
                me,
                Some(id),
                "host",
                "server.create",
                json!({"server_id":id}),
            )
            .await?;
            Ok(json!({"server_id":id,"job_id":result["job_id"],"state":"queued"}))
        }
        ServerStart { id } => {
            server_permission(db, me, *id, false).await?;
            wake(db, me, *id).await
        }
        ServerStop { id } => {
            server_permission(db, me, *id, false).await?;
            let kind: String =
                sqlx::query_scalar("SELECT kind FROM servers WHERE id=$1 FOR UPDATE")
                    .bind(id)
                    .fetch_one(&mut *db)
                    .await?;
            if kind == "lobby" {
                return Err(Error::conflict(
                    "The lobby stays running. Use the host maintenance procedure to stop it.",
                ));
            }
            let busy:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM jobs WHERE server_id=$1 AND worker='official' AND state IN ('queued','leased','waiting'))").bind(id).fetch_one(&mut *db).await?;
            if busy {
                return Err(Error::conflict(
                    "Wait for world operations to finish before stopping.",
                ));
            }
            sqlx::query("UPDATE servers SET desired='stopped',maintenance=true WHERE id=$1")
                .bind(id)
                .execute(&mut *db)
                .await?;
            let queued = job(
                db,
                me,
                Some(*id),
                "host",
                "server.stop",
                json!({"server_id":id}),
            )
            .await?;
            sqlx::query("UPDATE servers SET maintenance_job_id=$2 WHERE id=$1 AND maintenance_job_id IS NULL").bind(id).bind(super::services::uuid(&queued,"job_id")?).execute(&mut *db).await?;
            Ok(queued)
        }
        ServerJoin { id } => {
            // Commands already hold the account lock. Bind to the live session while
            // holding its row, including a game adapter's originally verified session.
            let session = sqlx::query(
                "SELECT * FROM game_sessions WHERE account_id=$1 AND lease_until>now() FOR UPDATE",
            )
            .bind(me)
            .fetch_optional(&mut *db)
            .await?
            .ok_or_else(|| {
                Error::conflict("Connect to lkjsxc.com:25591 first and wait in the lobby.")
            })?;
            let session_id: Uuid = session.get("session_id");
            if actor
                .session_hash
                .strip_prefix("game:")
                .is_some_and(|value| Uuid::parse_str(value).ok() != Some(session_id))
            {
                return Err(Error::conflict(
                    "Your game session has changed. Choose the destination again.",
                ));
            }
            can_join(db, me, *id).await?;
            crate::world::not_in_combat(db, me).await?;
            let server = sqlx::query("SELECT name,capabilities FROM servers WHERE id=$1")
                .bind(id)
                .fetch_one(&mut *db)
                .await?;
            let capabilities: Value = server.get("capabilities");
            if capabilities["proxy_join"] != true {
                return Err(Error::conflict(
                    "This server does not yet support joining through the lobby. Check its version, mods, and connection method.",
                ));
            }
            if session.get::<String, _>("client") == "bedrock" && capabilities["bedrock"] != true {
                return Err(Error::conflict(
                    "This server does not support Bedrock players.",
                ));
            }
            let pending = sqlx::query("SELECT * FROM jobs WHERE actor=$1 AND kind='player.join' AND state IN ('queued','waiting','leased') ORDER BY created_at FOR UPDATE")
                .bind(me).fetch_all(&mut *db).await?;
            for previous in &pending {
                let observed: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM game_session_history h WHERE h.account_id=$1 AND h.session_id=$2 AND h.server_id=$3 AND h.profile_id=$4 AND h.last_seen_at >= $5 AND h.last_seen_at <= now() AND h.started_at <= h.last_seen_at)")
                    .bind(me).bind(session_id).bind(previous.get::<Option<Uuid>,_>("server_id"))
                    .bind(session.get::<Uuid,_>("profile_id")).bind(previous.get::<chrono::DateTime<chrono::Utc>,_>("created_at"))
                    .fetch_one(&mut *db).await?;
                if previous.get::<Value, _>("payload")["session_id"] == json!(session_id)
                    && (observed
                        || session.get::<Option<Uuid>, _>("server_id")
                            == previous.get::<Option<Uuid>, _>("server_id"))
                    && previous.get::<Option<Uuid>, _>("server_id") != Some(*id)
                {
                    return Err(Error::conflict(
                        "Your arrival is still being confirmed. Wait before choosing another destination.",
                    ));
                }
                if previous.get::<Value, _>("payload")["session_id"] == json!(session_id)
                    && previous.get::<Option<Uuid>, _>("server_id") == Some(*id)
                {
                    return Ok(
                        json!({"job_id":previous.get::<Uuid,_>("id"),"state":previous.get::<String,_>("state"),"coalesced":true,"server_name":server.get::<String,_>("name")}),
                    );
                }
                if previous.get::<Value, _>("payload")["session_id"] == json!(session_id)
                    && previous.get::<Value, _>("progress")["phase"] == "connecting"
                {
                    return Err(Error::conflict(
                        "A transfer is already connecting. Wait for arrival or failure before choosing another destination.",
                    ));
                }
            }
            let mut superseded = Vec::new();
            for previous in pending {
                if previous.get::<Value, _>("payload")["session_id"] == json!(session_id) {
                    superseded.push(previous.get::<Value, _>("payload")["server_name"].clone());
                }
                sqlx::query("UPDATE jobs SET state='cancelled',error='A newer destination or game session replaced this request.',result=$2,progress=progress||'{\"phase\":\"cancelled\"}'::jsonb,lease_until=NULL,updated_at=now() WHERE id=$1")
                    .bind(previous.get::<Uuid,_>("id")).bind(json!({"effect":"none","reason":"superseded"})).execute(&mut *db).await?;
            }
            sqlx::query("UPDATE game_sessions SET pending_server_id=NULL,route_expires_at=NULL WHERE account_id=$1 AND session_id=$2")
                .bind(me).bind(session_id).execute(&mut *db).await?;
            // Bound the global proxy queue, including sleeping destinations.
            sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended('player.join.queue',0))")
                .execute(&mut *db)
                .await?;
            let queued: i64 = sqlx::query_scalar("SELECT count(*) FROM jobs WHERE kind='player.join' AND state IN ('queued','waiting','leased')")
                .fetch_one(&mut *db).await?;
            if queued >= 256 {
                return Err(Error::conflict(
                    "The travel queue is full. Stay connected and try again shortly.",
                ));
            }
            wake_for_join(db, me, *id).await?;
            let mut result = job(
                db,
                me,
                Some(*id),
                "proxy",
                "player.join",
                json!({"server_id":id,"server_name":server.get::<String,_>("name"),
                    "session_id":session_id,"native_uuid":session.get::<Uuid,_>("native_uuid"),
                    "profile_id":session.get::<Uuid,_>("profile_id"),"superseded":superseded}),
            )
            .await?;
            result["server_name"] = json!(server.get::<String, _>("name"));
            Ok(result)
        }
        ServerConfigure {
            id,
            name,
            visibility: v,
        } => {
            server_permission(db, me, *id, true).await?;
            visibility(v)?;
            sqlx::query("UPDATE servers SET name=$2,visibility=$3 WHERE id=$1")
                .bind(id)
                .bind(label(name, 64)?)
                .bind(v)
                .execute(db)
                .await?;
            Ok(json!({"updated":true}))
        }
        ServerMember { id, member, role } => {
            server_permission(db, me, *id, true).await?;
            let owner: Option<Uuid> =
                sqlx::query_scalar("SELECT owner FROM servers WHERE id=$1 FOR UPDATE")
                    .bind(id)
                    .fetch_one(&mut *db)
                    .await?;
            if Some(*member) == owner {
                return Err(Error::conflict(
                    "The server owner cannot be removed or demoted.",
                ));
            }
            if let Some(role) = role {
                if !matches!(role.as_str(), "guest" | "operator" | "administrator") {
                    return Err(Error::invalid("The role is invalid."));
                }
                sqlx::query("INSERT INTO server_members(server_id,account_id,role) VALUES($1,$2,$3) ON CONFLICT(server_id,account_id) DO UPDATE SET role=$3").bind(id).bind(member).bind(role).execute(&mut *db).await?;
            } else {
                sqlx::query("DELETE FROM server_members WHERE server_id=$1 AND account_id=$2")
                    .bind(id)
                    .bind(member)
                    .execute(&mut *db)
                    .await?;
            }
            audit(
                db,
                me,
                "server.member",
                id,
                json!({"member":member,"role":role}),
            )
            .await?;
            Ok(json!({"updated":true}))
        }
        ServerConsole { id, line } => {
            server_permission(db, me, *id, true).await?;
            let line = label(line, 1024)?;
            if line.contains(['\n', '\r', '\0']) {
                return Err(Error::invalid("Send console commands one line at a time."));
            }
            let kind: String = sqlx::query_scalar("SELECT kind FROM servers WHERE id=$1")
                .bind(id)
                .fetch_one(&mut *db)
                .await?;
            if kind != "custom" && !actor.admin {
                return Err(Error::forbidden());
            }
            audit(db, me, "server.console", id, json!({"command":line})).await?;
            job(
                db,
                me,
                Some(*id),
                "host",
                "server.console",
                json!({"line":line}),
            )
            .await
        }
        ServerLogs { .. }
        | ServerFiles { .. }
        | ServerFileRead { .. }
        | ServerFileWrite { .. }
        | ServerDirectoryCreate { .. }
        | ServerFileDelete { .. }
        | ServerOperator { .. } => crate::server_tools::command(db, actor, command).await,
        ServerInstall { id, artifact, path } => {
            server_permission(db, me, *id, true).await?;
            safe_path(path)?;
            let row=sqlx::query("SELECT a.id,a.kind,s.kind AS server_kind,s.desired,s.observed FROM artifacts a JOIN servers s ON s.id=a.server_id WHERE a.id=$1 AND a.server_id=$2").bind(artifact).bind(id).fetch_optional(&mut *db).await?.ok_or_else(Error::missing)?;
            if row.get::<String, _>("kind") != "world" && crate::server_tools::world_data(path) {
                return Err(Error::invalid(
                    "Individual world data files are protected. Upload a complete world archive.",
                ));
            }
            if row.get::<String, _>("server_kind") != "custom" {
                return Err(Error::forbidden());
            }
            if row.get::<String, _>("observed") != "stopped"
                || row.get::<String, _>("desired") != "stopped"
            {
                return Err(Error::conflict("Stop the server before applying files."));
            }
            job(
                db,
                me,
                Some(*id),
                "host",
                "server.install",
                json!({"artifact_id":artifact,"path":path}),
            )
            .await
        }
        ServerBackup { id } => {
            server_permission(db, me, *id, true).await?;
            let kind: String = sqlx::query_scalar("SELECT kind FROM servers WHERE id=$1")
                .bind(id)
                .fetch_one(&mut *db)
                .await?;
            if kind == "official" {
                return Err(Error::invalid(
                    "Use an official backup to save official server data.",
                ));
            }
            let backup = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO backups(id,server_id,kind,state) VALUES($1,$2,'server','queued')",
            )
            .bind(backup)
            .bind(id)
            .execute(&mut *db)
            .await?;
            job(
                db,
                me,
                Some(*id),
                "host",
                "server.backup",
                json!({"backup_id":backup}),
            )
            .await
        }
        ServerRestore { id, backup } => {
            server_permission(db, me, *id, true).await?;
            let exists:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM backups b JOIN servers s ON s.id=b.server_id WHERE b.id=$1 AND s.id=$2 AND b.kind='server' AND b.state='ready' AND s.kind='custom' AND s.observed='stopped' AND s.desired='stopped')").bind(backup).bind(id).fetch_one(&mut *db).await?;
            if !exists {
                return Err(Error::conflict(
                    "Select a completed backup belonging to this stopped personal server.",
                ));
            }
            audit(db, me, "server.restore", id, json!({"backup":backup})).await?;
            job(
                db,
                me,
                Some(*id),
                "host",
                "server.restore",
                json!({"backup_id":backup}),
            )
            .await
        }
        OfficialBackup => {
            if !actor.admin {
                return Err(Error::forbidden());
            }
            let server = crate::world::official_server(db).await?;
            crate::services::queue_official_backup(db, me, server, None).await
        }
        BackupPin { id, pinned } => {
            if !actor.admin {
                return Err(Error::forbidden());
            }
            let changed = sqlx::query(
                "UPDATE backups SET pinned=$2 WHERE id=$1 AND kind='official' AND state='ready'",
            )
            .bind(id)
            .bind(pinned)
            .execute(&mut *db)
            .await?
            .rows_affected();
            if changed != 1 {
                return Err(Error::conflict(
                    "Choose a completed official backup that has not started pruning.",
                ));
            }
            audit(db, me, "backup.pin", id, json!({"pinned":pinned})).await?;
            Ok(json!({"backup_id":id,"pinned":pinned}))
        }
        _ => Err(Error::invalid("This is not a server management action.")),
    }
}

pub async fn upload(
    State(app): State<App>,
    actor: Actor,
    Path(server): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<Value>> {
    let mut db = app.db.begin().await?;
    crate::deployment::enter(&mut db).await?;
    server_permission(&mut db, actor.id, server, true).await?;
    let quota: i64 = sqlx::query_scalar(
        "SELECT storage_mib*1024*1024 FROM servers WHERE id=$1 AND kind='custom'",
    )
    .bind(server)
    .fetch_optional(&mut *db)
    .await?
    .ok_or_else(Error::forbidden)?;
    let mut field = multipart
        .next_field()
        .await
        .map_err(|_| Error::invalid("The file could not be read."))?
        .ok_or_else(|| Error::invalid("Choose a file."))?;
    let name = field
        .file_name()
        .ok_or_else(|| Error::invalid("The file name is missing."))?
        .to_string();
    safe_path(&name)?;
    if name.contains('/') {
        return Err(Error::invalid("File names cannot contain /."));
    }
    let id = Uuid::new_v4();
    let temp = app
        .config
        .storage
        .join("artifacts")
        .join(format!("{id}.upload"));
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .await
        .map_err(Error::internal)?;
    let outcome=async {
        let mut digest=Sha256::new();let mut size=0_i64;
        while let Some(chunk)=field.chunk().await.map_err(|_|Error::invalid("The upload was interrupted."))? {
            size+=chunk.len() as i64;if size>quota || size>1024*1024*1024 {return Err(Error::invalid("The file exceeds the size limit."));}
            digest.update(&chunk);file.write_all(&chunk).await.map_err(Error::internal)?;
        }
        if size==0 {return Err(Error::invalid("Empty files cannot be saved."));}
        file.sync_all().await.map_err(Error::internal)?;drop(file);
        let mut tx=db;server_permission(&mut tx,actor.id,server,true).await?;
        sqlx::query("SELECT id FROM servers WHERE id=$1 FOR UPDATE").bind(server).fetch_one(&mut *tx).await?;
        let used:i64=sqlx::query_scalar("SELECT coalesce(sum(bytes),0)::bigint FROM artifacts WHERE server_id=$1").bind(server).fetch_one(&mut *tx).await?;
        if used+size>quota {return Err(Error::conflict("Stored files exceed the server’s storage allowance. Remove unneeded files first."));}
        let digest=hex::encode(digest.finalize());
        let kind=if name.ends_with(".jar"){"jar"}else if name.ends_with(".zip")||name.ends_with(".tar.gz"){"world"}else{"file"};
        // Object name is generated by the service and is never derived from the submitted filename.
        tokio::fs::rename(&temp,app.config.storage.join("artifacts").join(id.to_string())).await.map_err(Error::internal)?;
        sqlx::query("INSERT INTO artifacts(id,server_id,owner,sha256,bytes,name,kind) VALUES($1,$2,$3,$4,$5,$6,$7)").bind(id).bind(server).bind(actor.id).bind(&digest).bind(size).bind(&name).bind(kind).execute(&mut *tx).await?;
        tx.commit().await?;Ok(Json(json!({"id":id,"sha256":digest,"bytes":size,"name":name,"kind":kind})))
    }.await;
    if outcome.is_err() {
        let _ = tokio::fs::remove_file(temp).await;
    }
    outcome
}
