mod game;
mod settlement;
mod voice;
pub use game::{
    game_command, game_connect, game_disconnect, game_event, game_heartbeat, game_profile,
    game_view,
};
pub use voice::voice_token;

use crate::{
    App,
    auth::hash,
    error::{Error, Result},
};
use axum::{
    Json,
    extract::{FromRequestParts, Path, State},
    http::{header, request::Parts},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::Row;
use uuid::Uuid;

#[derive(Clone)]
pub struct Service {
    pub id: Uuid,
    pub role: String,
    pub server_id: Option<Uuid>,
}
impl Service {
    pub fn require(&self, role: &str) -> Result<()> {
        if self.role != role {
            return Err(Error::forbidden());
        }
        Ok(())
    }
}
impl FromRequestParts<App> for Service {
    type Rejection = Error;
    async fn from_request_parts(parts: &mut Parts, app: &App) -> Result<Self> {
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(Error::unauthorized)?;
        let row=sqlx::query("UPDATE service_credentials SET last_seen_at=now() WHERE token_hash=$1 AND revoked_at IS NULL RETURNING id,role,server_id").bind(hash(token)).fetch_optional(&app.db).await?.ok_or_else(Error::unauthorized)?;
        Ok(Self {
            id: row.get("id"),
            role: row.get("role"),
            server_id: row.get("server_id"),
        })
    }
}
pub async fn poll(State(app): State<App>, service: Service) -> Result<Json<Value>> {
    let mut tx = app.db.begin().await?;
    let row=sqlx::query("SELECT j.id FROM jobs j LEFT JOIN servers s ON s.id=j.server_id WHERE j.worker=$1 AND (j.server_id=$2 OR $1 IN ('host','proxy')) AND (j.state IN ('queued','waiting') OR j.state='leased' AND j.lease_until<now()) AND ($1 NOT IN ('official','lobby') OR s.observed='running') ORDER BY j.created_at FOR UPDATE OF j SKIP LOCKED LIMIT 1")
        .bind(&service.role).bind(service.server_id).fetch_optional(&mut *tx).await?;
    let Some(row) = row else {
        return Ok(Json(json!({"job":null})));
    };
    let lease = Uuid::new_v4();
    let id: Uuid = row.get("id");
    let value:Value=sqlx::query_scalar("UPDATE jobs SET state='leased',lease_token=$2,lease_owner=$3,lease_until=now()+interval '90 seconds',attempts=attempts+1,updated_at=now() WHERE id=$1 RETURNING to_jsonb(jobs)").bind(id).bind(lease).bind(service.id).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(json!({"job":value})))
}
#[derive(Deserialize)]
pub struct Ack {
    lease_token: Uuid,
    state: String,
    #[serde(default)]
    progress: Value,
    #[serde(default)]
    result: Value,
    error: Option<String>,
}
pub async fn ack(
    State(app): State<App>,
    service: Service,
    Path(id): Path<Uuid>,
    Json(request): Json<Ack>,
) -> Result<Json<Value>> {
    let mut tx = app.db.begin().await?;
    let row = sqlx::query(
        "SELECT * FROM jobs WHERE id=$1 AND lease_owner=$2 AND lease_token=$3 FOR UPDATE",
    )
    .bind(id)
    .bind(service.id)
    .bind(request.lease_token)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| {
        Error::conflict("ジョブの実行権が更新されています。結果を再照合してください。")
    })?;
    let old_state: String = row.get("state");
    if matches!(old_state.as_str(), "succeeded" | "failed" | "cancelled") {
        if matches!(request.state.as_str(), "succeeded" | "failed")
            && (old_state != request.state || row.get::<Value, _>("result") != request.result)
        {
            return Err(Error::conflict(
                "確定済みの結果とワールドの保存記録が一致しません。自動補償せず、管理者による照合が必要です。",
            ));
        }
        return Ok(Json(json!({"id":id,"state":old_state})));
    }
    if old_state != "leased" {
        return Err(Error::conflict("現在はこのジョブを更新できません。"));
    }
    if !matches!(
        request.state.as_str(),
        "leased" | "waiting" | "succeeded" | "failed"
    ) {
        return Err(Error::invalid("ジョブの状態が不正です。"));
    }
    let kind: String = row.get("kind");
    let actor: Uuid = row.get("actor");
    let payload: Value = row.get("payload");
    let server: Option<Uuid> = row.get("server_id");
    if request.state == "succeeded" {
        settlement::success(&mut tx, id, actor, server, &kind, &payload, &request.result).await?;
    } else if request.state == "failed" {
        if service.role == "official"
            && !matches!(
                request.result.get("effect").and_then(Value::as_str),
                Some("none" | "rolled_back")
            )
        {
            return Err(Error::conflict(
                "ワールド変更の回復を完了するまで、ジョブを失敗確定できません。",
            ));
        }
        settlement::failure(&mut tx, id, actor, server, &kind, &payload).await?;
    }
    // A capture preview is persisted before requesting each pet owner's consent, before removal.
    if kind == "asset.capture" && request.state == "leased" {
        if let Some(manifest) = request.progress.get("manifest") {
            let asset = uuid(&payload, "asset_id")?;
            let digest = hash(&serde_json::to_string(manifest).map_err(Error::internal)?);
            sqlx::query("UPDATE assets SET manifest=$2,manifest_sha256=$3 WHERE id=$1 AND state='capturing' AND job_id=$4").bind(asset).bind(manifest).bind(&digest).bind(id).execute(&mut *tx).await?;
        }
    }
    let error = request
        .error
        .map(|s| s.chars().take(2000).collect::<String>());
    sqlx::query("UPDATE jobs SET state=$2,progress=$3,result=CASE WHEN $2 IN ('succeeded','failed') THEN $4 ELSE result END,error=$5,lease_until=CASE WHEN $2='leased' THEN now()+interval '90 seconds' ELSE NULL END,updated_at=now() WHERE id=$1")
        .bind(id).bind(&request.state).bind(request.progress).bind(request.result).bind(error).execute(&mut *tx).await?;
    if matches!(request.state.as_str(), "succeeded" | "failed") {
        crate::commands::notify(
            &mut tx,
            actor,
            "job_finished",
            json!({"id":id,"kind":kind,"state":request.state}),
        )
        .await?;
    }
    tx.commit().await?;
    Ok(Json(json!({"id":id,"state":request.state})))
}
pub(super) fn uuid(value: &Value, key: &str) -> Result<Uuid> {
    value
        .get(key)
        .and_then(Value::as_str)
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| Error::invalid(format!("結果の {key} が不正です。")))
}
pub(super) fn receipt(result: &Value) -> Result<()> {
    if result.get("effect").and_then(Value::as_str) != Some("committed") {
        return Err(Error::invalid("永続化済みの処理結果が必要です。"));
    }
    Ok(())
}

#[derive(Deserialize)]
pub struct Observation {
    pub server_id: Option<Uuid>,
    pub observed: String,
    #[serde(default)]
    pub players: i32,
    #[serde(default)]
    pub metrics: Value,
    pub address: Option<String>,
    pub capabilities: Option<Value>,
}
pub async fn observe(
    State(app): State<App>,
    service: Service,
    Json(request): Json<Observation>,
) -> Result<Json<Value>> {
    if service.role != "host" && service.role != "proxy" && request.server_id != service.server_id {
        return Err(Error::forbidden());
    }
    if request.players < 0
        || !matches!(
            request.observed.as_str(),
            "running" | "stopped" | "starting" | "stopping" | "unknown" | "error"
        )
    {
        return Err(Error::invalid("観測した状態が不正です。"));
    }
    let mut tx = app.db.begin().await?;
    sqlx::query("INSERT INTO observations(credential,payload) VALUES($1,$2) ON CONFLICT(credential) DO UPDATE SET payload=$2,observed_at=now()").bind(service.id).bind(&request.metrics).execute(&mut *tx).await?;
    if let Some(server) = request.server_id {
        sqlx::query("UPDATE servers SET observed=$2,players=$3,last_observed_at=now(),empty_since=CASE WHEN $3>0 THEN NULL ELSE coalesce(empty_since,now()) END,address=coalesce($4,address) WHERE id=$1").bind(server).bind(request.observed).bind(request.players).bind(request.address).execute(&mut *tx).await?;
        if let Some(capabilities) = request.capabilities {
            if !matches!(service.role.as_str(), "host" | "official" | "lobby")
                || !capabilities.is_object()
            {
                return Err(Error::forbidden());
            }
            sqlx::query("UPDATE servers SET capabilities=$2 WHERE id=$1")
                .bind(server)
                .bind(capabilities)
                .execute(&mut *tx)
                .await?;
        }
    }
    tx.commit().await?;
    Ok(Json(json!({"recorded":true})))
}
pub async fn projection(State(app): State<App>, service: Service) -> Result<Json<Value>> {
    let mut result = json!({"role":service.role});
    if matches!(service.role.as_str(), "host" | "proxy") {
        result["servers"] = sqlx::query_scalar::<_, Value>(
            "SELECT coalesce(jsonb_agg(to_jsonb(s)),'[]') FROM servers s",
        )
        .fetch_one(&app.db)
        .await?;
    }
    if matches!(service.role.as_str(), "proxy" | "official" | "lobby") {
        result["sessions"]=sqlx::query_scalar::<_,Value>("SELECT coalesce(jsonb_agg(jsonb_build_object('account_id',g.account_id,'native_uuid',g.native_uuid,'profile_id',g.profile_id,'session_id',g.session_id,'server_id',g.server_id,'combat_until',g.combat_until,'name',p.name)),'[]') FROM game_sessions g JOIN principals p ON p.id=g.account_id WHERE g.lease_until>now() AND ($1::text='proxy' OR g.server_id=$2)").bind(&service.role).bind(service.server_id).fetch_one(&app.db).await?;
    }
    if matches!(service.role.as_str(), "official" | "lobby") {
        result["worlds"] = sqlx::query_scalar::<_, Value>(
            "SELECT coalesce(jsonb_agg(to_jsonb(w)),'[]') FROM worlds w WHERE server_id=$1",
        )
        .bind(service.server_id)
        .fetch_one(&app.db)
        .await?;
    }
    if service.role == "official" {
        result["claims"]=sqlx::query_scalar::<_,Value>("SELECT coalesce(jsonb_agg(to_jsonb(c)||jsonb_build_object('native_uuid',(SELECT native_uuid FROM profiles WHERE account_id=c.owner AND status='active'),'members',(SELECT coalesce(jsonb_agg(jsonb_build_object('account_id',m.account_id,'native_uuid',p.native_uuid,'can_build',m.can_build OR m.can_administer OR m.account_id=t.leader)),'[]') FROM team_members m JOIN teams t ON t.id=m.team_id LEFT JOIN profiles p ON p.account_id=m.account_id AND p.status='active' WHERE m.team_id=c.owner))),'[]') FROM claims c JOIN worlds w ON w.id=c.world_id WHERE w.server_id=$1 AND c.state<>'released'").bind(service.server_id).fetch_one(&app.db).await?;
        result["assets"]=sqlx::query_scalar::<_,Value>("SELECT coalesce(jsonb_agg(to_jsonb(a)),'[]') FROM assets a JOIN jobs j ON j.id=a.job_id WHERE j.server_id=$1 AND (a.state IN ('capturing','placing','quarantined') OR a.kind='land' AND a.state IN ('escrowed','listed'))")
            .bind(service.server_id).fetch_one(&app.db).await?;
        result["adventures"]=sqlx::query_scalar::<_,Value>("SELECT coalesce(jsonb_agg(to_jsonb(a)),'[]') FROM adventures a WHERE state NOT IN ('closed','refunded')").fetch_one(&app.db).await?;
        result["consents"]=sqlx::query_scalar::<_,Value>("SELECT coalesce(jsonb_agg(to_jsonb(c)),'[]') FROM asset_consents c JOIN assets a ON a.id=c.asset_id WHERE a.state='capturing'").fetch_one(&app.db).await?;
        result["paused"] = sqlx::query_scalar::<_, Value>(
            "SELECT value FROM settings WHERE key='official_mutations_paused'",
        )
        .fetch_one(&app.db)
        .await?;
    }
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct WorldIdentity {
    id: Uuid,
    name: String,
    native_uuid: Uuid,
}
pub async fn world_ready(
    State(app): State<App>,
    service: Service,
    Json(worlds): Json<Vec<WorldIdentity>>,
) -> Result<Json<Value>> {
    if !matches!(service.role.as_str(), "official" | "lobby") || worlds.len() > 4096 {
        return Err(Error::forbidden());
    }
    let mut tx = app.db.begin().await?;
    for world in worlds {
        let changed=sqlx::query("UPDATE worlds SET native_uuid=$4 WHERE id=$1 AND server_id=$2 AND name=$3 AND enabled AND (native_uuid IS NULL OR native_uuid=$4)")
            .bind(world.id).bind(service.server_id).bind(world.name).bind(world.native_uuid).execute(&mut *tx).await?.rows_affected();
        if changed != 1 {
            return Err(Error::conflict(
                "登録されたワールドのIDと実ファイルが一致しません。復旧または配置を確認してください。",
            ));
        }
    }
    tx.commit().await?;
    Ok(Json(json!({"registered":true})))
}
pub async fn artifact(
    State(app): State<App>,
    service: Service,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    service.require("host")?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM artifacts WHERE id=$1)")
        .bind(id)
        .fetch_one(&app.db)
        .await?;
    if !exists {
        return Err(Error::missing());
    }
    let file = tokio::fs::File::open(app.config.storage.join("artifacts").join(id.to_string()))
        .await
        .map_err(Error::internal)?;
    Ok((
        [(header::CONTENT_TYPE, "application/octet-stream")],
        axum::body::Body::from_stream(tokio_util::io::ReaderStream::new(file)),
    )
        .into_response())
}
pub async fn maintenance(app: App) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
    loop {
        interval.tick().await;
        if let Err(e) = tick(&app).await {
            tracing::error!(error=%e,"maintenance failed");
        }
    }
}
async fn tick(app: &App) -> Result<()> {
    let mut tx = app.db.begin().await?;
    let lock: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock(918740021)")
        .fetch_one(&mut *tx)
        .await?;
    if !lock {
        return Ok(());
    }
    let expired=sqlx::query("SELECT id,owner FROM adventures WHERE state='active' AND expires_at<=now() FOR UPDATE SKIP LOCKED").fetch_all(&mut *tx).await?;
    for row in expired {
        let id: Uuid = row.get("id");
        let owner: Uuid = row.get("owner");
        let server = crate::world::official_server(&mut tx).await?;
        crate::hosting::wake(&mut tx, owner, server).await?;
        sqlx::query("UPDATE adventures SET state='closing' WHERE id=$1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        crate::commands::job(
            &mut tx,
            owner,
            Some(server),
            "official",
            "adventure.close",
            json!({"adventure_id":id}),
        )
        .await?;
    }
    let idle=sqlx::query("SELECT s.id,coalesce(s.owner,(SELECT id FROM accounts WHERE administrator ORDER BY created_at LIMIT 1)) AS actor FROM servers s WHERE s.kind IN ('official','custom') AND s.observed='running' AND s.desired='running' AND s.players=0 AND s.empty_since<now()-interval '10 minutes' AND s.last_observed_at>now()-interval '45 seconds' AND NOT s.maintenance AND NOT EXISTS(SELECT 1 FROM jobs j WHERE j.server_id=s.id AND j.state IN ('queued','leased','waiting')) AND NOT EXISTS(SELECT 1 FROM adventures a WHERE s.kind='official' AND a.state IN ('preparing','activating','active','closing','refunding')) FOR UPDATE OF s SKIP LOCKED").fetch_all(&mut *tx).await?;
    for row in idle {
        let Some(actor) = row.get::<Option<Uuid>, _>("actor") else {
            continue;
        };
        let id: Uuid = row.get("id");
        sqlx::query("UPDATE servers SET desired='stopped' WHERE id=$1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        crate::commands::job(
            &mut tx,
            actor,
            Some(id),
            "host",
            "server.stop",
            json!({"idle":true}),
        )
        .await?;
    }
    sqlx::query("DELETE FROM oidc_flows WHERE expires_at<now()")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM sessions WHERE expires_at<now()")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    voice::revoke(app).await?;
    Ok(())
}
