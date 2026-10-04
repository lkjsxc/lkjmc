use super::join_state;
use super::{Service, uuid};
use crate::{
    App,
    auth::{Actor, create_account},
    error::{Error, Result},
};
use axum::extract::Path;
use axum::{Json, extract::State};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::{PgConnection, Row};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Connect {
    issuer: String,
    subject: String,
    display_name: String,
    native_uuid: Uuid,
    session_id: Uuid,
}
pub async fn game_connect(
    State(app): State<App>,
    service: Service,
    Json(mut request): Json<Connect>,
) -> Result<Json<Value>> {
    service.require("proxy")?;
    if !matches!(request.issuer.as_str(), "java" | "bedrock") || request.subject.len() > 40 {
        return Err(Error::invalid("text.the_game_identity_is_invalid"));
    }
    if request.issuer == "java"
        && Uuid::parse_str(&request.subject).ok() != Some(request.native_uuid)
    {
        return Err(Error::invalid("text.the_java_identity_does_not_match"));
    }
    if request.issuer == "bedrock"
        && request
            .subject
            .parse::<u64>()
            .ok()
            .filter(|n| *n > 0)
            .is_none()
    {
        return Err(Error::invalid("text.the_xuid_is_invalid"));
    }
    request.subject = if request.issuer == "java" {
        request.native_uuid.to_string()
    } else {
        request.subject.parse::<u64>().unwrap().to_string()
    };
    let mut tx = app.db.begin().await?;
    crate::deployment::enter(&mut tx).await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("game:{}:{}", request.issuer, request.subject))
        .execute(&mut *tx)
        .await?;
    let existing: Option<Uuid> =
        sqlx::query_scalar("SELECT account_id FROM identities WHERE issuer=$1 AND subject=$2")
            .bind(&request.issuer)
            .bind(&request.subject)
            .fetch_optional(&mut *tx)
            .await?;
    let account = if let Some(id) = existing {
        id
    } else {
        if request.issuer == "bedrock"
            && request.native_uuid
                != Uuid::from_u128(request.subject.parse::<u64>().unwrap() as u128)
        {
            return Err(Error::conflict(
                "text.an_external_linking_configuration_is_applied_to_this_un_568b172ab8",
            ));
        }
        let id = create_account(&mut tx, &request.display_name).await?;
        sqlx::query(
            "INSERT INTO identities(issuer,subject,account_id,display_name) VALUES($1,$2,$3,$4)",
        )
        .bind(&request.issuer)
        .bind(&request.subject)
        .bind(id)
        .bind(&request.display_name)
        .execute(&mut *tx)
        .await?;
        sqlx::query("UPDATE profiles SET native_uuid=$2 WHERE account_id=$1 AND status='active'")
            .bind(id)
            .bind(request.native_uuid)
            .execute(&mut *tx)
            .await?;
        id
    };
    let allowed:bool=sqlx::query_scalar("SELECT merged_into IS NULL AND (banned_until IS NULL OR banned_until<now()) FROM accounts WHERE id=$1 FOR UPDATE").bind(account).fetch_one(&mut *tx).await?;
    if !allowed {
        return Err(Error::forbidden());
    }
    let profile = crate::world::profile(&mut tx, account).await?;
    let native: Option<Uuid> = sqlx::query_scalar("SELECT native_uuid FROM profiles WHERE id=$1")
        .bind(profile)
        .fetch_one(&mut *tx)
        .await?;
    if native != Some(request.native_uuid) {
        return Err(Error::conflict(
            "text.the_game_identity_link_has_not_been_applied_contact_an_d9dd81e8f8",
        ));
    }
    sqlx::query("UPDATE identities SET display_name=$3 WHERE issuer=$1 AND subject=$2")
        .bind(&request.issuer)
        .bind(&request.subject)
        .bind(&request.display_name)
        .execute(&mut *tx)
        .await?;
    let row=sqlx::query("INSERT INTO game_sessions(account_id,profile_id,native_uuid,session_id,lease_until,client,combat_until) VALUES($1,$2,$3,$4,now()+interval '45 seconds',$5,(SELECT combat_until FROM accounts WHERE id=$1)) ON CONFLICT(account_id) DO UPDATE SET session_id=$4,profile_id=$2,native_uuid=$3,server_id=NULL,lease_until=now()+interval '45 seconds',client=$5,pending_server_id=NULL,route_expires_at=NULL,combat_until=greatest(game_sessions.combat_until,EXCLUDED.combat_until) WHERE game_sessions.lease_until<=now() OR game_sessions.session_id=$4 RETURNING account_id").bind(account).bind(profile).bind(request.native_uuid).bind(request.session_id).bind(&request.issuer).fetch_optional(&mut *tx).await?;
    if row.is_none() {
        return Err(Error::conflict(
            "text.this_account_is_already_connected_to_the_game_wait_for_d39031ffb4",
        ));
    }
    let language: String = sqlx::query_scalar("SELECT language FROM accounts WHERE id=$1")
        .bind(account)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(
        json!({"account_id":account,"profile_id":profile,"native_uuid":native,"session_id":request.session_id,"language":language,"client":request.issuer}),
    ))
}
#[derive(Deserialize)]
pub struct Heartbeat {
    account_id: Uuid,
    session_id: Uuid,
    server_id: Option<Uuid>,
    #[serde(default)]
    recovery: bool,
    #[serde(default)]
    join_job_id: Option<Uuid>,
    #[serde(default)]
    lease_token: Option<Uuid>,
    #[serde(default)]
    join_phase: Option<String>,
    #[serde(default)]
    error: Option<String>,
}
pub async fn game_heartbeat(
    State(app): State<App>,
    service: Service,
    Json(request): Json<Heartbeat>,
) -> Result<Json<Value>> {
    service.require("proxy")?;
    let mut tx = app.db.begin().await?;
    sqlx::query("SELECT id FROM accounts WHERE id=$1 FOR UPDATE")
        .bind(request.account_id)
        .execute(&mut *tx)
        .await?;
    let allowed:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM accounts a JOIN profiles p ON p.account_id=a.id AND p.status='active' WHERE a.id=$1 AND a.merged_into IS NULL AND (a.banned_until IS NULL OR a.banned_until<now()))").bind(request.account_id).fetch_one(&mut *tx).await?;
    if !allowed {
        return Err(Error::forbidden());
    }
    if let Some(server) = request.server_id {
        let present:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM game_sessions WHERE account_id=$1 AND session_id=$2 AND server_id=$3)").bind(request.account_id).bind(request.session_id).bind(server).fetch_one(&mut *tx).await?;
        if present {
            crate::hosting::can_remain(&mut tx, request.account_id, server).await?;
        } else {
            crate::hosting::can_join(&mut tx, request.account_id, server).await?;
        }
    }
    let n=sqlx::query("UPDATE game_sessions SET lease_until=now()+interval '45 seconds',server_id=$3,pending_server_id=CASE WHEN pending_server_id=$3 THEN NULL ELSE pending_server_id END WHERE account_id=$1 AND session_id=$2 AND lease_until>now()").bind(request.account_id).bind(request.session_id).bind(request.server_id).execute(&mut *tx).await?.rows_affected();
    if n == 0 {
        return Err(Error::conflict(
            "text.your_game_session_has_expired_reconnect_to_the_lobby",
        ));
    }
    if let Some(server) = request.server_id {
        sqlx::query("INSERT INTO game_session_history(session_id,server_id,account_id,profile_id) SELECT session_id,$3,account_id,profile_id FROM game_sessions WHERE account_id=$1 AND session_id=$2 ON CONFLICT(session_id,server_id) DO UPDATE SET last_seen_at=now()")
            .bind(request.account_id).bind(request.session_id).bind(server).execute(&mut *tx).await?;
    }
    // Enforce the total deadline even while a job is queued, not only when a
    // worker gets around to polling it. Connecting effects still need a fence.
    let overdue = sqlx::query("SELECT * FROM jobs WHERE actor=$1 AND kind='player.join' AND payload->>'session_id'=$2 AND state IN ('queued','waiting','leased') AND created_at <= now()-interval '10 minutes' AND coalesce(progress->>'phase','') <> 'connecting' FOR UPDATE")
        .bind(request.account_id).bind(request.session_id.to_string()).fetch_all(&mut *tx).await?;
    if !overdue.is_empty() {
        let session =
            sqlx::query("SELECT * FROM game_sessions WHERE account_id=$1 AND session_id=$2")
                .bind(request.account_id)
                .bind(request.session_id)
                .fetch_optional(&mut *tx)
                .await?;
        for job in overdue {
            join_state::finish(&mut tx, &job, session.as_ref(), "failed", Some("Server startup timed out after 10 minutes. Check the server status, then choose the destination again.")).await?;
        }
    }
    let ended: Vec<Value> = sqlx::query_scalar("SELECT jsonb_build_object('id',id,'payload',payload,'state',state,'error',error,'server_id',server_id) FROM jobs WHERE actor=$1 AND kind='player.join' AND payload->>'session_id'=$2 AND state='failed' AND updated_at > now()-interval '10 minutes' ORDER BY updated_at DESC LIMIT 32")
        .bind(request.account_id).bind(request.session_id.to_string()).fetch_all(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(json!({"alive":true,"join_results":ended})))
}
pub async fn game_disconnect(
    State(app): State<App>,
    service: Service,
    Json(request): Json<Heartbeat>,
) -> Result<Json<Value>> {
    service.require("proxy")?;
    let mut tx = app.db.begin().await?;
    sqlx::query("SELECT id FROM accounts WHERE id=$1 FOR UPDATE")
        .bind(request.account_id)
        .execute(&mut *tx)
        .await?;
    let departed =
        sqlx::query("DELETE FROM game_sessions WHERE account_id=$1 AND session_id=$2 RETURNING *")
            .bind(request.account_id)
            .bind(request.session_id)
            .fetch_optional(&mut *tx)
            .await?;
    let ended = sqlx::query("SELECT * FROM jobs WHERE actor=$1 AND kind='player.join' AND payload->>'session_id'=$2 AND state IN ('queued','waiting','leased') FOR UPDATE")
        .bind(request.account_id).bind(request.session_id.to_string()).fetch_all(&mut *tx).await?;
    for job in ended {
        join_state::finish(
            &mut tx,
            &job,
            departed.as_ref(),
            "failed",
            Some(
                "The original game session ended. Choose the destination again after reconnecting.",
            ),
        )
        .await?;
    }
    tx.commit().await?;
    Ok(Json(json!({"disconnected":true})))
}

/// Recheck access and client compatibility at the actual connection boundary,
/// including connections requested by other proxy plugins or stale Web jobs.
pub async fn game_route(
    State(app): State<App>,
    service: Service,
    Json(request): Json<Heartbeat>,
) -> Result<Json<Value>> {
    service.require("proxy")?;
    if request.join_phase.is_some() {
        return join_route(&app, &service, request).await;
    }
    let target = request
        .server_id
        .ok_or_else(|| Error::invalid("text.the_destination_is_missing"))?;
    let mut tx = app.db.begin().await?;
    let session = sqlx::query("SELECT g.* FROM game_sessions g JOIN accounts a ON a.id=g.account_id JOIN profiles p ON p.id=g.profile_id WHERE g.account_id=$1 AND g.session_id=$2 AND g.lease_until>now() AND a.merged_into IS NULL AND (a.banned_until IS NULL OR a.banned_until<now()) AND p.status='active' FOR UPDATE OF g")
        .bind(request.account_id).bind(request.session_id).fetch_optional(&mut *tx).await?.ok_or_else(Error::forbidden)?;
    crate::hosting::can_join(&mut tx, request.account_id, target).await?;
    let server = sqlx::query("SELECT * FROM servers WHERE id=$1")
        .bind(target)
        .fetch_one(&mut *tx)
        .await?;
    if request.recovery {
        if server.get::<String, _>("kind") != "lobby" {
            return Err(Error::forbidden());
        }
    } else if !(server.get::<String, _>("kind") == "lobby"
        && session.get::<Option<Uuid>, _>("server_id").is_none())
    {
        crate::world::not_in_combat(&mut tx, request.account_id).await?;
    }
    let capabilities: Value = server.get("capabilities");
    if capabilities["proxy_join"] != true {
        return Err(Error::conflict(
            "text.joining_this_server_through_the_lobby_has_not_been_verified_yet",
        ));
    }
    if session.get::<String, _>("client") == "bedrock" && capabilities["bedrock"] != true {
        return Err(Error::conflict(
            "text.this_server_does_not_support_bedrock_players",
        ));
    }
    let ready = server.get::<String, _>("observed") == "running"
        && server
            .get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_observed_at")
            .is_some_and(|at| at > chrono::Utc::now() - chrono::Duration::seconds(30));
    if ready {
        sqlx::query("UPDATE game_sessions SET pending_server_id=$3,route_expires_at=now()+interval '20 seconds' WHERE account_id=$1 AND session_id=$2")
            .bind(request.account_id).bind(request.session_id).bind(target).execute(&mut *tx).await?;
    } else {
        crate::hosting::wake_for_join(&mut tx, request.account_id, target).await?;
    }
    tx.commit().await?;
    Ok(Json(
        json!({"ready":ready,"server_id":target,"address":server.get::<Option<String>,_>("address"),"kind":server.get::<String,_>("kind")}),
    ))
}

/// Join control shares the route endpoint so the trusted proxy can fence a leased
/// request immediately before connecting and settle only its observed destination.
async fn join_route(app: &App, service: &Service, request: Heartbeat) -> Result<Json<Value>> {
    let phase = request.join_phase.as_deref().unwrap();
    if !matches!(
        phase,
        "check" | "connect" | "complete" | "fail" | "fence" | "cancel"
    ) {
        return Err(Error::invalid("text.the_join_phase_is_invalid"));
    }
    let mut tx = app.db.begin().await?;
    sqlx::query("SELECT id FROM accounts WHERE id=$1 FOR UPDATE")
        .bind(request.account_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(Error::forbidden)?;
    let session = sqlx::query("SELECT g.* FROM game_sessions g JOIN accounts a ON a.id=g.account_id JOIN profiles p ON p.id=g.profile_id WHERE g.account_id=$1 AND g.session_id=$2 AND ($3 OR (g.lease_until>now() AND a.merged_into IS NULL AND (a.banned_until IS NULL OR a.banned_until<now()) AND p.status='active')) FOR UPDATE OF g")
        .bind(request.account_id).bind(request.session_id).bind(matches!(phase,"complete"|"fail"|"fence")).fetch_optional(&mut *tx).await?;
    if phase == "cancel" {
        if session.is_none() {
            return Err(Error::forbidden());
        }
        let jobs = sqlx::query("SELECT * FROM jobs WHERE actor=$1 AND kind='player.join' AND payload->>'session_id'=$2 AND state IN ('queued','waiting','leased') FOR UPDATE")
            .bind(request.account_id).bind(request.session_id.to_string()).fetch_all(&mut *tx).await?;
        if jobs
            .iter()
            .any(|j| j.get::<Value, _>("progress")["phase"] == "connecting")
        {
            return Err(Error::conflict(
                "text.a_transfer_is_already_connecting_wait_for_arrival_or_fa_89c1d8ae7f",
            ));
        }
        let mut count = 0;
        for job in jobs {
            if join_state::finish(
                &mut tx,
                &job,
                session.as_ref(),
                "cancelled",
                Some("Travel was cancelled."),
            )
            .await?
                == "cancelled"
            {
                count += 1;
            }
        }
        sqlx::query("UPDATE game_sessions SET pending_server_id=NULL,route_expires_at=NULL WHERE account_id=$1 AND session_id=$2")
            .bind(request.account_id).bind(request.session_id).execute(&mut *tx).await?;
        tx.commit().await?;
        return Ok(Json(json!({"cancelled":count})));
    }
    let job_id = request.join_job_id.ok_or_else(Error::forbidden)?;
    let token = request.lease_token.ok_or_else(Error::forbidden)?;
    let job = sqlx::query("SELECT * FROM jobs WHERE id=$1 AND actor=$2 AND worker='proxy' AND kind='player.join' AND lease_owner=$3 AND lease_token=$4 FOR UPDATE")
        .bind(job_id).bind(request.account_id).bind(service.id).bind(token).fetch_optional(&mut *tx).await?.ok_or_else(Error::forbidden)?;
    let payload: Value = job.get("payload");
    let target = request.server_id.ok_or_else(Error::forbidden)?;
    if payload["session_id"] != json!(request.session_id)
        || job.get::<Option<Uuid>, _>("server_id") != Some(target)
    {
        return Err(Error::forbidden());
    }
    let state: String = job.get("state");
    if matches!(state.as_str(), "succeeded" | "failed" | "cancelled") {
        return Ok(Json(
            json!({"state":state,"ready":false,"error":job.get::<Option<String>,_>("error")}),
        ));
    }
    // Reconciliation cannot initiate a connection and remains safe after lease expiry.
    // A changed owner/token is still rejected above. Only a live lease may connect.
    if state != "leased"
        || (!matches!(phase, "complete" | "fail" | "fence")
            && job
                .get::<Option<chrono::DateTime<chrono::Utc>>, _>("lease_until")
                .is_none_or(|at| at <= chrono::Utc::now()))
    {
        return Err(Error::conflict(
            "text.the_job_lease_has_changed_recheck_the_result",
        ));
    }
    if matches!(phase, "fail" | "complete" | "fence") {
        let arrived = join_state::observed(&mut tx, &job, session.as_ref()).await?;
        if phase == "complete" && !arrived {
            return Err(Error::conflict(
                "text.the_destination_has_not_been_observed_in_this_game_session",
            ));
        }
        // `fence` is the trusted proxy's acknowledgement that the original
        // connection future has naturally completed (never Future.cancel()).
        // Route or lease expiry alone does not prove that a network effect stopped.
        if !arrived && phase == "fail" && job.get::<Value, _>("progress")["phase"] == "connecting" {
            return Err(Error::conflict(
                "text.the_connection_must_be_fenced_before_travel_can_fail",
            ));
        }
        let error = request
            .error
            .as_ref()
            .map(|e| e.chars().take(2000).collect::<String>());
        let state =
            join_state::finish(&mut tx, &job, session.as_ref(), "failed", error.as_deref()).await?;
        sqlx::query("UPDATE game_sessions SET pending_server_id=NULL,route_expires_at=NULL WHERE account_id=$1 AND session_id=$2")
            .bind(request.account_id).bind(request.session_id).execute(&mut *tx).await?;
        tx.commit().await?;
        return Ok(Json(json!({"state":state})));
    }
    let session = session.ok_or_else(|| {
        Error::conflict("text.your_game_session_has_changed_choose_the_destination_again")
    })?;
    if payload["native_uuid"] != json!(session.get::<Uuid, _>("native_uuid"))
        || payload["profile_id"] != json!(session.get::<Uuid, _>("profile_id"))
    {
        return Err(Error::forbidden());
    }
    if job.get::<chrono::DateTime<chrono::Utc>, _>("created_at")
        < chrono::Utc::now() - chrono::Duration::minutes(10)
        && session.get::<Option<Uuid>, _>("server_id") != Some(target)
    {
        return Err(Error::conflict(
            "text.server_startup_timed_out_stay_in_the_lobby_check_the_se_6300e9f11e",
        ));
    }
    crate::hosting::can_join(&mut tx, request.account_id, target).await?;
    crate::world::not_in_combat(&mut tx, request.account_id).await?;
    let server = sqlx::query("SELECT * FROM servers WHERE id=$1")
        .bind(target)
        .fetch_one(&mut *tx)
        .await?;
    let capabilities: Value = server.get("capabilities");
    if capabilities["proxy_join"] != true
        || session.get::<String, _>("client") == "bedrock" && capabilities["bedrock"] != true
    {
        return Err(Error::conflict(
            "text.this_server_does_not_support_this_game_client_through_the_lobby",
        ));
    }
    let ready = server.get::<String, _>("observed") == "running"
        && server
            .get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_observed_at")
            .is_some_and(|at| at > chrono::Utc::now() - chrono::Duration::seconds(30));
    // wake() clears server.error when retrying; expose a failed start first.
    if !ready && server.get::<Option<String>, _>("error").is_some() {
        return Err(Error::conflict(
            "text.server_startup_failed_check_the_server_status_or_ask_it_39329d57e3",
        ));
    }
    let startup: Option<Value> = sqlx::query_scalar("SELECT jsonb_build_object('state',state,'progress',progress) FROM jobs WHERE server_id=$1 AND kind IN ('server.create','server.start','server.restore') ORDER BY created_at DESC LIMIT 1")
        .bind(target).fetch_optional(&mut *tx).await?;
    if !ready && startup.as_ref().is_some_and(|s| s["state"] == "failed") {
        return Err(Error::conflict(
            "text.server_startup_failed_check_the_server_status_or_ask_it_39329d57e3",
        ));
    }
    if !ready {
        crate::hosting::wake_for_join(&mut tx, request.account_id, target).await?;
    }
    if phase == "connect" {
        if !ready {
            return Err(Error::conflict(
                "text.the_destination_is_not_ready_wait_in_the_lobby",
            ));
        }
        sqlx::query("UPDATE jobs SET progress=$2,updated_at=now() WHERE id=$1")
            .bind(job_id)
            .bind(json!({"phase":"connecting","server_name":server.get::<String,_>("name")}))
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE game_sessions SET pending_server_id=$3,route_expires_at=now()+interval '20 seconds' WHERE account_id=$1 AND session_id=$2")
            .bind(request.account_id).bind(request.session_id).bind(target).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(Json(
        json!({"state":"leased","ready":ready,"observed":server.get::<String,_>("observed"),"server_name":server.get::<String,_>("name"),"startup":startup}),
    ))
}

/// Floodgate's local linking extension reads only completed Core identities.
/// It cannot create links or restore an archived progression dataset.
pub async fn game_linked(
    State(app): State<App>,
    service: Service,
    Path(native_id): Path<Uuid>,
) -> Result<Json<Value>> {
    service.require("proxy")?;
    if native_id.as_u128() > u64::MAX as u128 {
        return Ok(Json(json!({"link":null})));
    }
    let subject = (native_id.as_u128() as u64).to_string();
    let link: Option<Value> = sqlx::query_scalar("SELECT jsonb_build_object('java_uuid',j.subject,'java_username',j.display_name,'bedrock_uuid',$2::text) FROM identities b JOIN identities j ON j.account_id=b.account_id AND j.issuer='java' JOIN profiles p ON p.account_id=b.account_id AND p.status='active' JOIN accounts a ON a.id=b.account_id WHERE b.issuer='bedrock' AND b.subject=$1 AND p.native_uuid::text=j.subject AND a.merged_into IS NULL AND (a.banned_until IS NULL OR a.banned_until<now())")
        .bind(subject).bind(native_id.to_string()).fetch_optional(&app.db).await?;
    Ok(Json(json!({"link":link})))
}
#[derive(Deserialize)]
pub struct GameCommand {
    account_id: Uuid,
    session_id: Uuid,
    #[serde(flatten)]
    request: crate::commands::Request,
}
pub async fn game_command(
    State(app): State<App>,
    service: Service,
    Json(request): Json<GameCommand>,
) -> Result<Json<Value>> {
    if !matches!(service.role.as_str(), "proxy" | "official" | "lobby") {
        return Err(Error::forbidden());
    }
    let admin:bool=sqlx::query_scalar("SELECT a.administrator FROM accounts a JOIN game_sessions g ON g.account_id=a.id WHERE a.id=$1 AND g.session_id=$2 AND g.lease_until>now() AND ($3::text='proxy' OR g.server_id=$4) AND (a.banned_until IS NULL OR a.banned_until<now())").bind(request.account_id).bind(request.session_id).bind(&service.role).bind(service.server_id).fetch_optional(&app.db).await?.ok_or_else(Error::forbidden)?;
    let actor = Actor {
        id: request.account_id,
        admin,
        csrf: String::new(),
        // ServerJoin rechecks this authenticated session under the command's account
        // lock. Web actors have hexadecimal session hashes, never this internal marker.
        session_hash: format!("game:{}", request.session_id),
    };
    crate::commands::execute(&app, &actor, request.request)
        .await
        .map(Json)
}
#[derive(Deserialize)]
pub struct GameEvent {
    id: Uuid,
    account_id: Uuid,
    session_id: Uuid,
    occurred_at: chrono::DateTime<chrono::Utc>,
    kind: String,
    payload: Value,
}

pub async fn game_profile(
    State(app): State<App>,
    service: Service,
    Path(native_id): Path<Uuid>,
) -> Result<Json<Value>> {
    if !matches!(service.role.as_str(), "official" | "lobby" | "proxy") {
        return Err(Error::forbidden());
    }
    let value:Value=sqlx::query_scalar("SELECT jsonb_build_object('account_id',g.account_id,'profile_id',g.profile_id,'session_id',g.session_id,'native_uuid',g.native_uuid,'server_id',g.server_id,'combat_until',g.combat_until,'name',p.name,'language',a.language,'client',g.client) FROM game_sessions g JOIN accounts a ON a.id=g.account_id JOIN profiles pr ON pr.id=g.profile_id JOIN principals p ON p.id=a.id WHERE g.native_uuid=$1 AND g.lease_until>now() AND a.merged_into IS NULL AND pr.status='active' AND (a.banned_until IS NULL OR a.banned_until<now()) AND ($2::uuid IS NULL OR g.server_id=$2 OR g.pending_server_id=$2 AND g.route_expires_at>now())")
        .bind(native_id).bind(service.server_id).fetch_optional(&app.db).await?.ok_or_else(Error::forbidden)?;
    if let Some(server) = service.server_id {
        let mut db = app.db.acquire().await?;
        crate::hosting::can_join(&mut db, uuid(&value, "account_id")?, server).await?;
    }
    Ok(Json(value))
}

#[derive(Deserialize)]
pub struct GameView {
    account_id: Uuid,
    session_id: Uuid,
    view: String,
    #[serde(default)]
    query: Value,
}
pub async fn game_view(
    State(app): State<App>,
    service: Service,
    Json(request): Json<GameView>,
) -> Result<Json<Value>> {
    if !matches!(service.role.as_str(), "official" | "lobby" | "proxy") {
        return Err(Error::forbidden());
    }
    let admin:bool=sqlx::query_scalar("SELECT a.administrator FROM accounts a JOIN game_sessions g ON g.account_id=a.id WHERE a.id=$1 AND g.session_id=$2 AND g.lease_until>now() AND ($3='proxy' OR g.server_id=$4) AND (a.banned_until IS NULL OR a.banned_until<now())")
        .bind(request.account_id).bind(request.session_id).bind(service.role).bind(service.server_id).fetch_optional(&app.db).await?.ok_or_else(Error::forbidden)?;
    let actor = Actor {
        id: request.account_id,
        admin,
        csrf: String::new(),
        session_hash: String::new(),
    };
    match request.view.as_str() {
        "me" => crate::queries::me(State(app), actor).await,
        "players" => {
            crate::queries::players(
                State(app),
                actor,
                axum::extract::Query(
                    serde_json::from_value(request.query)
                        .map_err(|_| Error::invalid("text.the_search_parameters_are_invalid"))?,
                ),
            )
            .await
        }
        "messages" => {
            crate::queries::messages(
                State(app),
                actor,
                Path(uuid(&request.query, "room")?),
                axum::extract::Query(
                    serde_json::from_value(request.query)
                        .map_err(|_| Error::invalid("text.the_search_parameters_are_invalid"))?,
                ),
            )
            .await
        }
        "job" => crate::queries::job(State(app), actor, Path(uuid(&request.query, "id")?)).await,
        "report_preview" => {
            crate::queries::report_preview(
                State(app),
                actor,
                Json(
                    serde_json::from_value(request.query)
                        .map_err(|_| Error::invalid("text.the_report_target_is_invalid"))?,
                ),
            )
            .await
        }
        _ => crate::queries::view(State(app), actor, Path(request.view)).await,
    }
}
pub async fn game_event(
    State(app): State<App>,
    service: Service,
    Json(request): Json<GameEvent>,
) -> Result<Json<Value>> {
    service.require("official")?;
    let mut tx = app.db.begin().await?;
    crate::deployment::enter(&mut tx).await?;
    // A persisted outbox can replay after the account has merged. Validate its
    // original event before checking today's profile, then acknowledge without minting again.
    if let Some(row) =
        sqlx::query("SELECT account_id,kind,payload,credential FROM game_events WHERE id=$1")
            .bind(request.id)
            .fetch_optional(&mut *tx)
            .await?
    {
        let credential: Uuid = row.get("credential");
        let same_server:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM service_credentials WHERE id=$1 AND server_id=$2 AND role='official')").bind(credential).bind(service.server_id).fetch_one(&mut *tx).await?;
        if !same_server
            || row.get::<Uuid, _>("account_id") != request.account_id
            || row.get::<String, _>("kind") != request.kind
            || row.get::<Value, _>("payload") != request.payload
        {
            return Err(Error::conflict(
                "text.the_event_id_was_reused_with_different_content",
            ));
        }
        return Ok(Json(json!({"duplicate":true})));
    }
    let mut participants = vec![request.account_id];
    if request.kind == "combat" {
        participants.push(uuid(&request.payload, "target")?);
    }
    sqlx::query("SELECT id FROM accounts WHERE id=ANY($1) ORDER BY id FOR UPDATE")
        .bind(&participants)
        .fetch_all(&mut *tx)
        .await?;
    let exists:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM game_session_history h JOIN profiles p ON p.id=h.profile_id WHERE h.account_id=$1 AND h.server_id=$2 AND h.session_id=$3 AND $4 BETWEEN h.started_at-interval '5 seconds' AND h.last_seen_at+interval '45 seconds' AND $4<=now()+interval '5 seconds' AND p.account_id=$1 AND p.status IN ('active','moving'))")
        .bind(request.account_id).bind(service.server_id).bind(request.session_id).bind(request.occurred_at).fetch_one(&mut *tx).await?;
    if !exists {
        return Err(Error::forbidden());
    }
    let inserted=sqlx::query("INSERT INTO game_events(id,credential,account_id,kind,payload) VALUES($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING").bind(request.id).bind(service.id).bind(request.account_id).bind(&request.kind).bind(&request.payload).execute(&mut *tx).await?.rows_affected();
    if inserted == 0 {
        return Ok(Json(json!({"duplicate":true})));
    }
    match request.kind.as_str() {
        "block.placed" | "walk.distance" | "crop.harvest" | "adventure.completed" => {
            let amount = request.payload["amount"]
                .as_i64()
                .filter(|n| (1..=1000).contains(n))
                .ok_or_else(|| Error::invalid("text.the_achievement_increment_is_invalid"))?;
            reward_event(
                &mut tx,
                request.account_id,
                request.id,
                &request.kind,
                amount,
            )
            .await?;
        }
        "combat" => {
            sqlx::query("UPDATE accounts SET combat_until=greatest(combat_until,$2+interval '30 seconds') WHERE id=ANY($1)").bind(&participants).bind(request.occurred_at).execute(&mut *tx).await?;
            sqlx::query("UPDATE game_sessions SET combat_until=greatest(combat_until,$4+interval '30 seconds') WHERE account_id IN ($1,$2) AND server_id=$3").bind(request.account_id).bind(uuid(&request.payload,"target")?).bind(service.server_id).bind(request.occurred_at).execute(&mut *tx).await?;
        }
        "position" => {
            sqlx::query("UPDATE game_sessions SET last_position=$2 WHERE account_id=$1")
                .bind(request.account_id)
                .bind(request.payload)
                .execute(&mut *tx)
                .await?;
        }
        _ => return Err(Error::invalid("text.the_game_event_type_is_invalid")),
    }
    tx.commit().await?;
    Ok(Json(json!({"recorded":true})))
}
pub(super) async fn reward_event(
    db: &mut PgConnection,
    actor: Uuid,
    event_id: Uuid,
    event: &str,
    amount: i64,
) -> Result<()> {
    crate::economy::unpaused(db).await?;
    let rules = sqlx::query("SELECT * FROM achievements WHERE event=$1 ORDER BY key")
        .bind(event)
        .fetch_all(&mut *db)
        .await?;
    for rule in rules {
        let owner = if rule.get::<bool, _>("team") {
            sqlx::query_scalar::<_, Uuid>("SELECT team_id FROM team_members WHERE account_id=$1")
                .bind(actor)
                .fetch_optional(&mut *db)
                .await?
        } else {
            Some(actor)
        };
        let Some(owner) = owner else { continue };
        let key: String = rule.get("key");
        let target: i64 = rule.get("target");
        sqlx::query("INSERT INTO achievement_progress(owner,achievement) VALUES($1,$2) ON CONFLICT DO NOTHING").bind(owner).bind(&key).execute(&mut *db).await?;
        let row=sqlx::query("UPDATE achievement_progress SET progress=least($4,progress+$3) WHERE owner=$1 AND achievement=$2 AND earned_at IS NULL RETURNING progress").bind(owner).bind(&key).bind(amount).bind(target).fetch_optional(&mut *db).await?;
        if row.is_some_and(|r| r.get::<i64, _>("progress") >= target) {
            sqlx::query(
                "UPDATE achievement_progress SET earned_at=now() WHERE owner=$1 AND achievement=$2",
            )
            .bind(owner)
            .bind(&key)
            .execute(&mut *db)
            .await?;
            sqlx::query("UPDATE land_allowances SET chunks=chunks+$2 WHERE owner=$1")
                .bind(owner)
                .bind(rule.get::<i32, _>("land_chunks"))
                .execute(&mut *db)
                .await?;
            let coins: i64 = rule.get("coins");
            if coins > 0 {
                crate::economy::book(
                    db,
                    actor,
                    &format!("achievement:{owner}:{key}"),
                    "achievement",
                    json!({"achievement":key,"event_id":event_id}),
                    &[(owner, coins)],
                    true,
                )
                .await?;
            }
            crate::commands::notify(db,actor,"achievement",json!({"key":key,"owner":owner,"coins":coins,"land_chunks":rule.get::<i32,_>("land_chunks")})).await?;
        }
    }
    Ok(())
}
