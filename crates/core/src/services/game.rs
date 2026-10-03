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
        return Err(Error::invalid("The game identity is invalid."));
    }
    if request.issuer == "java"
        && Uuid::parse_str(&request.subject).ok() != Some(request.native_uuid)
    {
        return Err(Error::invalid("The Java identity does not match."));
    }
    if request.issuer == "bedrock"
        && request
            .subject
            .parse::<u64>()
            .ok()
            .filter(|n| *n > 0)
            .is_none()
    {
        return Err(Error::invalid("The XUID is invalid."));
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
                "An external linking configuration is applied to this unlinked Bedrock identity.",
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
            "The game identity link has not been applied. Contact an administrator.",
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
            "This account is already connected to the game. Wait for saving and disconnection to finish.",
        ));
    }
    let language: String = sqlx::query_scalar("SELECT language FROM accounts WHERE id=$1")
        .bind(account)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(
        json!({"account_id":account,"profile_id":profile,"native_uuid":native,"session_id":request.session_id,"language":language}),
    ))
}
#[derive(Deserialize)]
pub struct Heartbeat {
    account_id: Uuid,
    session_id: Uuid,
    server_id: Option<Uuid>,
    #[serde(default)]
    recovery: bool,
}
pub async fn game_heartbeat(
    State(app): State<App>,
    service: Service,
    Json(request): Json<Heartbeat>,
) -> Result<Json<Value>> {
    service.require("proxy")?;
    let mut tx = app.db.begin().await?;
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
            "Your game session has expired. Reconnect to the lobby.",
        ));
    }
    if let Some(server) = request.server_id {
        sqlx::query("INSERT INTO game_session_history(session_id,server_id,account_id,profile_id) SELECT session_id,$3,account_id,profile_id FROM game_sessions WHERE account_id=$1 AND session_id=$2 ON CONFLICT(session_id,server_id) DO UPDATE SET last_seen_at=now()")
            .bind(request.account_id).bind(request.session_id).bind(server).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(Json(json!({"alive":true})))
}
pub async fn game_disconnect(
    State(app): State<App>,
    service: Service,
    Json(request): Json<Heartbeat>,
) -> Result<Json<Value>> {
    service.require("proxy")?;
    sqlx::query("DELETE FROM game_sessions WHERE account_id=$1 AND session_id=$2")
        .bind(request.account_id)
        .bind(request.session_id)
        .execute(&app.db)
        .await?;
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
    let target = request
        .server_id
        .ok_or_else(|| Error::invalid("The destination is missing."))?;
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
            "Joining this server through the lobby has not been verified yet.",
        ));
    }
    if session.get::<String, _>("client") == "bedrock" && capabilities["bedrock"] != true {
        return Err(Error::conflict(
            "This server does not support Bedrock players.",
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
        crate::hosting::wake(&mut tx, request.account_id, target).await?;
    }
    tx.commit().await?;
    Ok(Json(
        json!({"ready":ready,"server_id":target,"address":server.get::<Option<String>,_>("address"),"kind":server.get::<String,_>("kind")}),
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
        session_hash: String::new(),
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
    let value:Value=sqlx::query_scalar("SELECT jsonb_build_object('account_id',g.account_id,'profile_id',g.profile_id,'session_id',g.session_id,'native_uuid',g.native_uuid,'server_id',g.server_id,'combat_until',g.combat_until,'name',p.name,'language',a.language) FROM game_sessions g JOIN accounts a ON a.id=g.account_id JOIN profiles pr ON pr.id=g.profile_id JOIN principals p ON p.id=a.id WHERE g.native_uuid=$1 AND g.lease_until>now() AND a.merged_into IS NULL AND pr.status='active' AND (a.banned_until IS NULL OR a.banned_until<now()) AND ($2::uuid IS NULL OR g.server_id=$2 OR g.pending_server_id=$2 AND g.route_expires_at>now())")
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
                        .map_err(|_| Error::invalid("The search parameters are invalid."))?,
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
                        .map_err(|_| Error::invalid("The search parameters are invalid."))?,
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
                        .map_err(|_| Error::invalid("The report target is invalid."))?,
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
                "The event ID was reused with different content.",
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
                .ok_or_else(|| Error::invalid("The achievement increment is invalid."))?;
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
        _ => return Err(Error::invalid("The game event type is invalid.")),
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
