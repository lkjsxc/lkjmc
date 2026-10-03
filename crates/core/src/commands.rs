use crate::{
    App,
    auth::{Actor, hash},
    error::{Error, Result},
};
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::Row;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub request_id: Uuid,
    pub command: Command,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    FriendRequest {
        target: Uuid,
    },
    FriendRespond {
        target: Uuid,
        accept: bool,
    },
    FriendRemove {
        target: Uuid,
    },
    DirectRoom {
        target: Uuid,
    },
    RoomCreate {
        name: String,
    },
    MessageSend {
        room: Uuid,
        body: String,
    },
    MessageDelete {
        id: i64,
    },
    RoomRead {
        room: Uuid,
    },
    RoomLeave {
        room: Uuid,
    },
    TeamCreate {
        name: String,
    },
    TeamPermissions {
        team: Uuid,
        member: Uuid,
        build: bool,
        sell: bool,
        spend: bool,
        members: bool,
        administer: bool,
    },
    TeamTransfer {
        team: Uuid,
        target: Uuid,
    },
    TeamLeave,
    TeamDisband {
        team: Uuid,
    },
    PartyCreate {
        name: String,
    },
    PartyReady {
        ready: bool,
    },
    PartyLeave,
    PartyTransfer {
        target: Uuid,
    },
    CommunityCreate {
        name: String,
    },
    Invite {
        kind: String,
        resource: Uuid,
        target: Uuid,
    },
    InviteRespond {
        id: Uuid,
        accept: bool,
    },
    Block {
        target: Uuid,
        blocked: bool,
    },
    Language {
        language: String,
    },
    Privacy {
        display_name: String,
        dm_policy: String,
        activity_policy: String,
    },
    NotificationsRead {
        through: i64,
    },
    Report {
        target: Option<Uuid>,
        reason: String,
        message_ids: Vec<i64>,
    },
    ReportResolve {
        id: Uuid,
        status: String,
        resolution: String,
    },
    Ban {
        target: Uuid,
        hours: i32,
        reason: String,
    },
    RankSet {
        target: Uuid,
        rank: i16,
    },
    RankConfigure {
        id: i16,
        name: String,
        server_count: i32,
        concurrent_servers: i32,
        memory_mib: i32,
        cpu_millis: i32,
        storage_mib: i64,
    },
    LinkBegin,
    LinkPresent {
        code: String,
    },
    LinkConfirm {
        id: Uuid,
        selected_profile: Uuid,
    },
    ServerCreate {
        name: String,
        software: String,
        version: String,
        memory_mib: i32,
        cpu_millis: i32,
        storage_mib: i64,
        visibility: String,
        community: Option<Uuid>,
    },
    ServerStart {
        id: Uuid,
    },
    ServerStop {
        id: Uuid,
    },
    ServerJoin {
        id: Uuid,
    },
    ServerConfigure {
        id: Uuid,
        name: String,
        visibility: String,
    },
    ServerMember {
        id: Uuid,
        member: Uuid,
        role: Option<String>,
    },
    ServerConsole {
        id: Uuid,
        line: String,
    },
    ServerLogs {
        id: Uuid,
    },
    ServerInstall {
        id: Uuid,
        artifact: Uuid,
        path: String,
    },
    ServerBackup {
        id: Uuid,
    },
    ServerRestore {
        id: Uuid,
        backup: Uuid,
    },
    ClaimCreate {
        owner: Option<Uuid>,
        name: String,
        min_x: i32,
        min_z: i32,
        max_x: i32,
        max_z: i32,
    },
    ClaimRelease {
        id: Uuid,
    },
    HomeSet {
        name: String,
    },
    HomeTravel {
        id: Uuid,
    },
    HomeDelete {
        id: Uuid,
    },
    TeleportRequest {
        target: Uuid,
    },
    AssetCapture {
        owner: Option<Uuid>,
        kind: String,
        title: String,
        selection: Value,
        include_contents: bool,
    },
    AssetConsent {
        id: Uuid,
        manifest_sha256: String,
    },
    AssetPlace {
        id: Uuid,
        placement: Value,
    },
    AssetReceive {
        id: Uuid,
    },
    AssetWithdraw {
        id: Uuid,
    },
    ListingCreate {
        asset: Uuid,
        price: i64,
    },
    ListingCancel {
        id: Uuid,
    },
    ListingBuy {
        id: Uuid,
        owner: Option<Uuid>,
    },
    WalletTransfer {
        owner: Option<Uuid>,
        target: Uuid,
        amount: i64,
    },
    NpcSell {
        material: String,
        amount: i32,
    },
    AdventureCreate,
    AdventureCancel {
        id: Uuid,
    },
    AdventureJoin {
        id: Uuid,
    },
    OfficialBackup,
    BackupPin {
        id: Uuid,
        pinned: bool,
    },
}

pub async fn http_command(
    State(app): State<App>,
    actor: Actor,
    Json(request): Json<Request>,
) -> Result<Json<Value>> {
    execute(&app, &actor, request).await.map(Json)
}
pub async fn execute(app: &App, actor: &Actor, request: Request) -> Result<Value> {
    let mut tx = app.db.begin().await?;
    crate::deployment::enter(&mut tx).await?;
    // Linking and transfers lock every participating account in the same order. A stale
    // HTTP/game authorization must not allow writes after a merge, ban, or demotion.
    let mut accounts = vec![actor.id];
    if let Command::LinkConfirm { id, .. } = &request.command {
        if let Some(other) = sqlx::query_scalar::<_, Uuid>(
            "SELECT candidate FROM link_requests WHERE id=$1 AND initiator=$2 AND candidate IS NOT NULL",
        ).bind(id).bind(actor.id).fetch_optional(&mut *tx).await? {
            accounts.push(other);
        }
    }
    if let Command::WalletTransfer { target, .. } = &request.command {
        accounts.push(*target);
    }
    let locked = sqlx::query("SELECT id,administrator,merged_into,banned_until>now() AS banned FROM accounts WHERE id=ANY($1) ORDER BY id FOR UPDATE")
        .bind(&accounts).fetch_all(&mut *tx).await?;
    let current = locked
        .iter()
        .find(|row| row.get::<Uuid, _>("id") == actor.id)
        .ok_or_else(Error::unauthorized)?;
    if current.get::<Option<Uuid>, _>("merged_into").is_some()
        || current.get::<Option<bool>, _>("banned") == Some(true)
    {
        return Err(Error::forbidden());
    }
    let actor = &Actor {
        admin: current.get("administrator"),
        ..actor.clone()
    };
    let encoded = serde_json::to_string(&request.command).map_err(Error::internal)?;
    let digest = hash(&encoded);
    if let Some(row) =
        sqlx::query("SELECT request_hash,response FROM idempotency WHERE actor=$1 AND key=$2")
            .bind(actor.id)
            .bind(request.request_id)
            .fetch_optional(&mut *tx)
            .await?
    {
        if row.get::<String, _>("request_hash") != digest {
            return Err(Error::conflict(
                "A request ID cannot be reused with different content.",
            ));
        }
        return Ok(row.get("response"));
    }
    crate::world::profile(&mut tx, actor.id).await?;
    let recent: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM idempotency WHERE actor=$1 AND created_at>now()-interval '1 minute'",
    )
    .bind(actor.id)
    .fetch_one(&mut *tx)
    .await?;
    if recent >= 120 {
        return Err(Error {
            status: axum::http::StatusCode::TOO_MANY_REQUESTS,
            code: "rate_limited",
            message: "Too many actions. Please wait a moment and try again.".into(),
        });
    }
    use Command::*;
    if let ServerCreate {
        software,
        version,
        storage_mib,
        ..
    } = &request.command
    {
        crate::presets::validate(app, software, version, *storage_mib)?;
    }
    let result = match &request.command {
        ServerCreate { .. }
        | ServerStart { .. }
        | ServerStop { .. }
        | ServerJoin { .. }
        | ServerConfigure { .. }
        | ServerMember { .. }
        | ServerConsole { .. }
        | ServerLogs { .. }
        | ServerInstall { .. }
        | ServerBackup { .. }
        | ServerRestore { .. }
        | OfficialBackup
        | BackupPin { .. } => crate::hosting::command(&mut tx, actor, &request.command).await?,
        ClaimCreate { .. }
        | ClaimRelease { .. }
        | HomeSet { .. }
        | HomeTravel { .. }
        | HomeDelete { .. }
        | TeleportRequest { .. }
        | AssetCapture { .. }
        | AssetConsent { .. }
        | AssetPlace { .. }
        | AssetReceive { .. }
        | AssetWithdraw { .. }
        | NpcSell { .. }
        | AdventureCreate
        | AdventureCancel { .. }
        | AdventureJoin { .. }
        | LinkBegin
        | LinkPresent { .. }
        | LinkConfirm { .. } => crate::world::command(&mut tx, actor, &request.command).await?,
        ListingCreate { .. } | ListingCancel { .. } | ListingBuy { .. } | WalletTransfer { .. } => {
            crate::economy::command(&mut tx, actor, &request.command, request.request_id).await?
        }
        _ => crate::social::command(&mut tx, actor, &request.command).await?,
    };
    let response = json!({"request_id":request.request_id,"result":result});
    sqlx::query("INSERT INTO idempotency(actor,key,request_hash,response) VALUES($1,$2,$3,$4)")
        .bind(actor.id)
        .bind(request.request_id)
        .bind(digest)
        .bind(&response)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(response)
}

pub fn label(value: &str, max: usize) -> Result<String> {
    let s = value.trim();
    if s.is_empty()
        || s.chars().count() > max
        || s.chars().any(|c| c.is_control() && c != '\n' && c != '\t')
    {
        return Err(Error::invalid(format!("Enter 1–{max} characters.")));
    }
    Ok(s.to_string())
}
pub async fn notify(
    db: &mut sqlx::PgConnection,
    account: Uuid,
    kind: &str,
    body: Value,
) -> Result<()> {
    sqlx::query("INSERT INTO notifications(account_id,kind,body) VALUES($1,$2,$3)")
        .bind(account)
        .bind(kind)
        .bind(body)
        .execute(db)
        .await?;
    Ok(())
}
pub async fn job(
    db: &mut sqlx::PgConnection,
    actor: Uuid,
    server: Option<Uuid>,
    worker: &str,
    kind: &str,
    payload: Value,
) -> Result<Value> {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO jobs(id,actor,server_id,worker,kind,payload) VALUES($1,$2,$3,$4,$5,$6)",
    )
    .bind(id)
    .bind(actor)
    .bind(server)
    .bind(worker)
    .bind(kind)
    .bind(payload)
    .execute(db)
    .await?;
    Ok(json!({"job_id":id,"state":"queued"}))
}
