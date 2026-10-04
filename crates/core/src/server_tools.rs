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

#[derive(Debug, serde::Serialize)]
pub struct ActionStatus {
    pub allowed: bool,
    pub reason: Option<&'static str>,
}
impl ActionStatus {
    fn from_reason(reason: Option<&'static str>) -> Self {
        Self {
            allowed: reason.is_none(),
            reason,
        }
    }
}
#[derive(Debug, serde::Serialize)]
pub struct ServerActions {
    pub join: ActionStatus,
    pub start: ActionStatus,
    pub stop: ActionStatus,
    pub logs: ActionStatus,
    pub files: ActionStatus,
}
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MachineState {
    Unknown,
    Running,
    Stopped,
}
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GameState {
    Unprovisioned,
    Unknown,
    Starting,
    Running,
    Stopping,
    Stopped,
    Error,
}
#[derive(Debug, serde::Serialize)]
pub struct ServerStatus {
    pub machine_state: MachineState,
    pub game_state: GameState,
    pub observation_fresh: bool,
    pub joinable: bool,
    pub activity: Option<String>,
    pub actions: ServerActions,
}
fn fresh(value: &Value) -> bool {
    value
        .as_str()
        .and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok())
        .is_some_and(|t| {
            t <= chrono::Utc::now() && t > chrono::Utc::now() - chrono::Duration::seconds(45)
        })
}
/// Presentation of observed facts and current access, never a replacement for effect-time authorization.
pub fn server_status(server: &Value) -> ServerStatus {
    let observation_fresh = fresh(&server["last_observed_at"]);
    let game = if server["observed"] == "unprovisioned" {
        "unprovisioned"
    } else if observation_fresh {
        server["observed"].as_str().unwrap_or("unknown")
    } else {
        "unknown"
    };
    let machine_state = match (
        fresh(&server["machine_observed_at"]),
        server["machine_observed"].as_str(),
    ) {
        (true, Some("running")) => MachineState::Running,
        (true, Some("stopped")) => MachineState::Stopped,
        _ => MachineState::Unknown,
    };
    let game_state = match game {
        "unprovisioned" => GameState::Unprovisioned,
        "running" => GameState::Running,
        "starting" => GameState::Starting,
        "stopping" => GameState::Stopping,
        "stopped" => GameState::Stopped,
        "error" => GameState::Error,
        _ => GameState::Unknown,
    };
    let inspecting = !server["inspection"].is_null();
    let operation = server["active_operation"]["kind"].as_str();
    let restoring = operation == Some("server.restore");
    let busy = !server["maintenance_job_id"].is_null() || operation.is_some();
    let maintenance = server["maintenance"] == true;
    let manage = server["can_manage"] == true;
    let administer = server["can_administer"] == true;
    let join_reason = if maintenance {
        Some("maintenance")
    } else if game == "unprovisioned" {
        Some("provisioning")
    } else if !observation_fresh {
        Some("observation_stale")
    } else if game == "stopped" {
        Some("server_sleeping")
    } else if game != "running" {
        Some("game_not_ready")
    } else {
        None
    };
    let start_reason = if !manage || inspecting && !administer {
        Some("permission_required")
    } else if busy || maintenance && !inspecting {
        Some("maintenance")
    } else if game == "unprovisioned" {
        Some("provisioning")
    } else if game == "running" {
        Some("already_running")
    } else {
        None
    };
    let stop_reason = if !manage {
        Some("permission_required")
    } else if server["kind"] == "lobby" {
        Some("lobby_always_running")
    } else if maintenance {
        Some("maintenance")
    } else if game == "stopped" {
        Some("already_stopped")
    } else {
        None
    };
    let logs_reason = if !manage {
        Some("permission_required")
    } else if restoring {
        Some("restore_in_progress")
    } else if !observation_fresh {
        Some("observation_stale")
    } else if game == "unprovisioned" {
        Some("provisioning")
    } else if game == "stopped" && server["inspection"]["guest_ready"] != true {
        Some("server_sleeping")
    } else {
        None
    };
    let files_reason = if !administer {
        Some("permission_required")
    } else if restoring {
        Some("restore_in_progress")
    } else if inspecting && server["inspection"]["guest_ready"] == true
        || game == "running" && observation_fresh
    {
        None
    } else if inspecting || maintenance {
        Some("maintenance")
    } else {
        Some("files_closed")
    };
    ServerStatus {
        machine_state,
        game_state,
        observation_fresh,
        joinable: game == "running" && join_reason.is_none(),
        activity: operation
            .map(str::to_owned)
            .or_else(|| inspecting.then(|| "server.inspection".into())),
        actions: ServerActions {
            join: ActionStatus::from_reason(
                if matches!(join_reason, Some("server_sleeping" | "game_not_ready")) {
                    None
                } else {
                    join_reason
                },
            ),
            start: ActionStatus::from_reason(start_reason),
            stop: ActionStatus::from_reason(stop_reason),
            logs: ActionStatus::from_reason(logs_reason),
            files: ActionStatus::from_reason(files_reason),
        },
    }
}
#[derive(Debug, serde::Serialize)]
pub struct OperationStatus {
    pub id: Option<Uuid>,
    pub kind: String,
    pub state: String,
    pub phase: Option<String>,
    pub terminal: bool,
    pub outcome: &'static str,
    pub automatic_retry: bool,
}
pub fn operation_status(job: &Value) -> OperationStatus {
    let state = job["state"].as_str().unwrap_or("unknown");
    let terminal = matches!(
        state,
        "succeeded" | "failed" | "cancelled" | "delivery_unknown"
    );
    let outcome = match state {
        "succeeded" => "completed",
        "failed" => "failed",
        "cancelled" => "cancelled",
        "delivery_unknown" => "delivery_unknown",
        _ => "pending",
    };
    OperationStatus {
        id: job["job_id"]
            .as_str()
            .or_else(|| job["id"].as_str())
            .and_then(|s| s.parse().ok()),
        kind: job["kind"].as_str().unwrap_or("").into(),
        state: state.into(),
        phase: job["progress"]["phase"].as_str().map(str::to_owned),
        terminal,
        outcome,
        automatic_retry: matches!(state, "queued" | "waiting" | "leased"),
    }
}

pub fn passive(kind: &str) -> bool {
    matches!(kind, "server.logs" | "server.files" | "server.file.read")
}
pub fn world_data(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or("");
    matches!(name, "level.dat" | "level.dat_old" | "session.lock")
        || [".dat", ".mca", ".mcr"].iter().any(|s| name.ends_with(s))
}
pub fn protected(path: &str) -> bool {
    static PATHS: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    let paths = PATHS.get_or_init(|| {
        serde_json::from_str(include_str!("../../../ops/guest/managed-paths.json"))
            .expect("managed path policy")
    });
    let path = path.to_ascii_lowercase();
    paths
        .iter()
        .any(|p| path == *p || path.starts_with(&format!("{p}/")))
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
    if let ServerInspection { id, open } = command {
        return inspection_command(db, actor, *id, *open).await;
    }
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
/// Non-renewable admission. The server row serializes this with wake and host context.
async fn inspection_command(
    db: &mut PgConnection,
    actor: &Actor,
    id: Uuid,
    open: bool,
) -> Result<Value> {
    hosting::server_permission(db, actor.id, id, true).await?;
    let server: Value =
        sqlx::query_scalar("SELECT to_jsonb(s) FROM servers s WHERE id=$1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *db)
            .await?
            .ok_or_else(Error::missing)?;
    if server["kind"] != "custom" {
        return Err(Error::conflict(
            "File inspection is only available on custom servers.",
        ));
    }
    let current = &server["inspection"];
    if !current.is_null() {
        if open && inspection_valid(db, &server).await? {
            return Ok(json!({"job_id":current["id"],"inspection":current}));
        }
        if open {
            return Err(Error::conflict(
                "The previous file session is closing. Reopen files after cleanup completes.",
            ));
        }
        return close_inspection(db, id, current, Some(actor.id)).await;
    }
    if !open {
        return Ok(json!({"inspection":null,"state":"closed"}));
    }
    stopped(db, id, false).await?;
    hosting::reserve_capacity(db, id).await?;
    let busy: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM jobs WHERE server_id=$1 AND worker='host' AND kind NOT IN ('server.logs','server.files','server.file.read') AND state IN ('queued','leased','waiting'))")
        .bind(id).fetch_one(&mut *db).await?;
    if busy || server["maintenance"] == true {
        return Err(Error::conflict(
            "Wait for the current server operation before opening files.",
        ));
    }
    let result = job(
        db,
        actor.id,
        Some(id),
        "host",
        "server.inspection",
        json!({"open":true,"server_name":server["name"]}),
    )
    .await?;
    let job_id = result["job_id"]
        .as_str()
        .unwrap()
        .parse::<Uuid>()
        .map_err(Error::internal)?;
    let inspection: Value = sqlx::query_scalar("UPDATE servers SET inspection=jsonb_build_object('id',$2::uuid,'actor',$3::uuid,'owner',owner,'state','opening','guest_ready',false,'expires_at',now()+interval '15 minutes','expires_unix',extract(epoch FROM now()+interval '15 minutes')::bigint),maintenance=true WHERE id=$1 RETURNING inspection")
        .bind(id).bind(job_id).bind(actor.id).fetch_one(&mut *db).await?;
    sqlx::query(
        "UPDATE jobs SET payload=payload||jsonb_build_object('inspection',$2::jsonb) WHERE id=$1",
    )
    .bind(job_id)
    .bind(&inspection)
    .execute(&mut *db)
    .await?;
    Ok(json!({"job_id":job_id,"state":"queued","inspection":inspection}))
}
async fn close_inspection(
    db: &mut PgConnection,
    id: Uuid,
    inspection: &Value,
    requested_by: Option<Uuid>,
) -> Result<Value> {
    if let Some(result) = sqlx::query_scalar::<_,Value>("SELECT jsonb_build_object('job_id',id,'state',state) FROM jobs WHERE server_id=$1 AND kind='server.inspection' AND payload->>'open'='false' AND payload->'inspection'->>'id'=$2 AND state IN ('queued','leased','waiting') LIMIT 1")
        .bind(id).bind(inspection["id"].as_str()).fetch_optional(&mut *db).await? { return Ok(result); }
    let actor = match requested_by {
        Some(actor) => actor,
        None => inspection["actor"]
            .as_str()
            .unwrap_or("")
            .parse::<Uuid>()
            .map_err(Error::internal)?,
    };
    let result = job(
        db,
        actor,
        Some(id),
        "host",
        "server.inspection",
        json!({"open":false,"inspection":inspection,"automatic":requested_by.is_none()}),
    )
    .await?;
    sqlx::query("UPDATE servers SET inspection=inspection||'{\"state\":\"closing\",\"guest_ready\":false}'::jsonb WHERE id=$1").bind(id).execute(&mut *db).await?;
    Ok(result)
}
/// Cleanup is authorized by the original window, even after its opener is removed.
pub async fn inspection_valid(db: &mut PgConnection, server: &Value) -> Result<bool> {
    let i = &server["inspection"];
    if i.is_null()
        || i["owner"] != server["owner"]
        || i["state"] == "closing"
        || i["expires_unix"].as_i64().unwrap_or(0) <= chrono::Utc::now().timestamp()
    {
        return Ok(false);
    }
    let actor = i["actor"]
        .as_str()
        .unwrap_or("")
        .parse::<Uuid>()
        .map_err(Error::internal)?;
    let active: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM accounts WHERE id=$1 AND merged_into IS NULL AND (banned_until IS NULL OR banned_until<now()))").bind(actor).fetch_one(&mut *db).await?;
    if !active {
        return Ok(false);
    }
    Ok(hosting::server_permission(
        db,
        actor,
        server["id"]
            .as_str()
            .unwrap()
            .parse()
            .map_err(Error::internal)?,
        true,
    )
    .await
    .is_ok())
}
async fn sweep_inspections(db: &mut PgConnection) -> Result<()> {
    let servers: Vec<Value> = sqlx::query_scalar(
        "SELECT to_jsonb(s) FROM servers s WHERE inspection IS NOT NULL FOR UPDATE SKIP LOCKED",
    )
    .fetch_all(&mut *db)
    .await?;
    for server in servers {
        if !inspection_valid(db, &server).await? {
            close_inspection(
                db,
                server["id"]
                    .as_str()
                    .unwrap()
                    .parse()
                    .map_err(Error::internal)?,
                &server["inspection"],
                None,
            )
            .await?;
        }
    }
    Ok(())
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
    let row:Option<(Value,bool)>=sqlx::query_as("SELECT jsonb_build_object('id',j.id,'kind',j.kind,'open',CASE WHEN j.kind='server.inspection' THEN j.payload->'open' ELSE NULL END,'server_id',j.server_id,'state',j.state,'progress',j.progress,'result',j.result,'error',j.error,'updated_at',j.updated_at), j.kind NOT IN ('server.logs','server.files','server.file.read') OR (j.updated_at>now()-interval '2 minutes' AND EXISTS(SELECT 1 FROM servers s JOIN accounts a ON a.id=$2 WHERE s.id=j.server_id AND a.merged_into IS NULL AND (a.banned_until IS NULL OR a.banned_until<now()) AND (s.owner=a.id OR a.administrator OR EXISTS(SELECT 1 FROM server_members m WHERE m.server_id=s.id AND m.account_id=a.id AND (m.role='administrator' OR j.kind='server.logs' AND m.role='operator')) OR EXISTS(SELECT 1 FROM community_members m WHERE m.community_id=s.community_id AND m.account_id=a.id AND m.administrator)))) FROM jobs j WHERE j.id=$1 AND (j.actor=$2 OR $3)").bind(id).bind(actor.id).bind(actor.admin).fetch_optional(&app.db).await?;
    let (mut value, allowed) = row.ok_or_else(Error::missing)?;
    if !allowed {
        return Err(Error::forbidden());
    }
    value["operation_status"] =
        serde_json::to_value(operation_status(&value)).map_err(Error::internal)?;
    crate::system_message::project_system_content(&mut value);
    Ok(Json(value))
}

/// Passive reads have independent bounded leases; mutation ownership remains serial per server.
pub async fn poll(
    State(app): State<App>,
    service: crate::services::Service,
    Json(request): Json<Value>,
) -> Result<Json<Value>> {
    if service.role != "host" {
        return crate::services::poll(State(app), service).await;
    }
    let read_lane = match request["lane"].as_str() {
        None | Some("mutation") => false,
        Some("read") => true,
        _ => return Err(Error::invalid("The host job lane is invalid.")),
    };
    let mut tx = app.db.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended('host-job-dispatch',0))")
        .execute(&mut *tx)
        .await?;
    prune(&mut tx).await?;
    sweep_inspections(&mut tx).await?;
    let sql = if read_lane {
        "SELECT j.id FROM jobs j JOIN servers s ON s.id=j.server_id WHERE j.worker='host' AND j.kind IN ('server.logs','server.files','server.file.read') AND (j.state='queued' OR j.state='waiting' AND j.updated_at<now()-interval '5 seconds' OR j.state='leased' AND j.lease_until<now()) AND (SELECT count(*) FROM jobs active WHERE active.worker='host' AND active.kind IN ('server.logs','server.files','server.file.read') AND active.state='leased' AND active.lease_until>now())<2 AND NOT EXISTS(SELECT 1 FROM jobs active WHERE active.server_id=j.server_id AND active.state='leased' AND active.lease_until>now() AND (active.kind='server.restore' OR active.kind IN ('server.logs','server.files','server.file.read'))) AND NOT EXISTS(SELECT 1 FROM jobs active WHERE active.id=s.maintenance_job_id AND active.kind='server.restore' AND active.state IN ('queued','leased','waiting')) ORDER BY j.updated_at,j.created_at FOR UPDATE OF j SKIP LOCKED LIMIT 1"
    } else {
        "SELECT j.id FROM jobs j JOIN servers s ON s.id=j.server_id WHERE j.worker='host' AND j.kind NOT IN ('server.logs','server.files','server.file.read') AND (j.state='queued' OR j.state='waiting' AND j.updated_at<now()-interval '5 seconds' OR j.state='leased' AND j.lease_until<now()) AND (s.maintenance_job_id IS NULL OR s.maintenance_job_id=j.id) AND (s.inspection IS NULL OR j.kind IN ('server.inspection','server.file.write','server.file.delete','server.directory.create','server.operator','server.install')) AND NOT EXISTS(SELECT 1 FROM jobs other WHERE other.id<>j.id AND other.worker='host' AND other.server_id=j.server_id AND other.state='leased' AND other.lease_until>now() AND (other.kind NOT IN ('server.logs','server.files','server.file.read') OR j.kind='server.restore')) ORDER BY CASE WHEN j.kind='server.inspection' AND j.payload->>'open'='false' THEN 0 WHEN j.kind='server.inspection' THEN 1 WHEN j.kind='official.backup.prune' THEN 1 ELSE 0 END,j.updated_at,j.created_at FOR UPDATE OF j SKIP LOCKED LIMIT 1"
    };
    let id: Option<Uuid> = sqlx::query_scalar(sql).fetch_optional(&mut *tx).await?;
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
        "server.inspection"
            | "server.file.write"
            | "server.file.delete"
            | "server.directory.create"
            | "server.operator"
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
        "server.inspection" => {
            if result["effect"] != "committed"
                || result["inspection_id"] != payload["inspection"]["id"]
                || result["open"] != payload["open"]
                || result["game_stopped"] != true
                || (payload["open"] == true
                    && result["guest_ready"] == true
                    && result["expires_unix"].as_i64().is_none_or(|at| {
                        at > payload["inspection"]["expires_unix"].as_i64().unwrap_or(0)
                    }))
            {
                return Err(invalid());
            }
        }
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
    if passive(&kind)
        && row
            .get::<Option<chrono::DateTime<chrono::Utc>>, _>("lease_until")
            .is_none_or(|t| t <= chrono::Utc::now())
    {
        return Err(Error::conflict("The read lease expired."));
    }
    if request.state == "leased"
        && row
            .get::<Option<chrono::DateTime<chrono::Utc>>, _>("lease_until")
            .is_none_or(|t| t <= chrono::Utc::now())
    {
        return Err(Error::conflict(
            "The job lease expired. Reconcile saved receipts under a fresh lease.",
        ));
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
    if kind == "server.inspection" && matches!(request.state.as_str(), "succeeded" | "failed") {
        let payload: Value = row.get("payload");
        let sid: Uuid = row.get("server_id");
        if payload["open"] == false && request.state == "succeeded" {
            sqlx::query("UPDATE servers SET inspection=NULL,maintenance=false,observed='stopped' WHERE id=$1 AND inspection->>'id'=$2")
                .bind(sid).bind(payload["inspection"]["id"].as_str()).execute(&mut *tx).await?;
        } else if payload["open"] == true {
            let state = if request.state == "succeeded" && request.result["guest_ready"] == true {
                "ready"
            } else {
                "closing"
            };
            sqlx::query("UPDATE servers SET inspection=inspection||jsonb_build_object('state',$3::text,'guest_ready',$3='ready','checked_at',now(),'expires_unix',least((inspection->>'expires_unix')::bigint,$4::bigint),'expires_at',to_timestamp(least((inspection->>'expires_unix')::bigint,$4::bigint))) WHERE id=$1 AND inspection->>'id'=$2 AND inspection->>'state'='opening'")
                .bind(sid).bind(payload["inspection"]["id"].as_str()).bind(state).bind(request.result["expires_unix"].as_i64().unwrap_or(chrono::Utc::now().timestamp())).execute(&mut *tx).await?;
        }
    }
    let error = request
        .error
        .map(|s| s.chars().take(2000).collect::<String>());
    sqlx::query("UPDATE jobs SET state=$2,progress=CASE WHEN $2='leased' AND $3='{}'::jsonb THEN progress ELSE $3 END,result=CASE WHEN $2 IN ('succeeded','failed') THEN $4 ELSE result END,error=$5,lease_until=CASE WHEN $2='leased' THEN now()+interval '90 seconds' ELSE NULL END,updated_at=now() WHERE id=$1").bind(id).bind(&request.state).bind(request.progress).bind(request.result).bind(error).execute(&mut *tx).await?;
    if matches!(request.state.as_str(), "succeeded" | "failed") {
        sqlx::query("UPDATE servers s SET maintenance=s.inspection IS NOT NULL OR EXISTS(SELECT 1 FROM jobs j WHERE j.server_id=s.id AND j.id<>$1 AND j.kind='server.stop' AND j.state IN ('queued','leased','waiting')),maintenance_job_id=NULL WHERE maintenance_job_id=$1").bind(id).execute(&mut *tx).await?;
        if !passive(&kind) && row.get::<Value, _>("payload")["automatic"] != true {
            let payload = row.get::<Value, _>("payload");
            let server_name: Option<String> =
                sqlx::query_scalar("SELECT name FROM servers WHERE id=$1")
                    .bind(row.get::<Option<Uuid>, _>("server_id"))
                    .fetch_optional(&mut *tx)
                    .await?;
            crate::commands::notify(
                &mut tx,
                row.get("actor"),
                "job_finished",
                json!({"id":id,"kind":kind,"state":request.state,"server_id":row.get::<Option<Uuid>,_>("server_id"),"server_name":server_name,"open":payload["open"],"path":payload["path"]}),
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
            "plugins/lkjmc/config.yml",
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
