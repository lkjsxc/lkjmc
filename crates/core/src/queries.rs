use crate::{
    App,
    auth::{Actor, audit},
    error::{Error, Result},
};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgConnection;
use uuid::Uuid;
mod teams;
pub use teams::{TeamQuery, detail as team};

pub async fn ready(State(app): State<App>) -> Result<Json<Value>> {
    sqlx::query("SELECT 1").execute(&app.db).await?;
    Ok(Json(
        json!({"ready":true,"version":env!("CARGO_PKG_VERSION"),"login_configured":app.config.oidc_issuer.is_some(),"deployment_gate_supported":true,"game_address":app.config.game_address}),
    ))
}
pub async fn me(State(app): State<App>, actor: Actor) -> Result<Json<Value>> {
    let account:Value=sqlx::query_scalar("SELECT jsonb_build_object('id',a.id,'name',p.name,'administrator',a.administrator,'language',a.language,'rank',to_jsonb(r),'dm_policy',a.dm_policy,'activity_policy',a.activity_policy,'profile',(SELECT to_jsonb(f) FROM profiles f WHERE f.account_id=a.id AND f.status<>'archived'),'identities',(SELECT coalesce(jsonb_agg(jsonb_build_object('issuer',i.issuer,'display_name',i.display_name)),'[]') FROM identities i WHERE i.account_id=a.id)) FROM accounts a JOIN principals p ON p.id=a.id JOIN trust_ranks r ON r.id=a.trust_rank WHERE a.id=$1").bind(actor.id).fetch_one(&app.db).await?;
    Ok(Json(
        json!({"account":account,"csrf":actor.csrf,"game_address":app.config.game_address,"voice_available":app.config.voice_url.is_some(),"development":app.config.development}),
    ))
}
async fn aggregate(app: &App, actor: Uuid, sql: &str) -> Result<Value> {
    Ok(sqlx::query_scalar::<_, Value>(sql)
        .bind(actor)
        .fetch_one(&app.db)
        .await?)
}
pub async fn page_view(
    State(app): State<App>,
    actor: Actor,
    Path(view): Path<String>,
    Query(query): Query<crate::pages::PageQuery>,
) -> Result<Json<Value>> {
    view_section(app, actor, &view, query.section.as_deref()).await
}
pub async fn view(
    State(app): State<App>,
    actor: Actor,
    Path(view): Path<String>,
) -> Result<Json<Value>> {
    view_section(app, actor, &view, None).await
}
pub async fn view_section(
    app: App,
    actor: Actor,
    view: &str,
    section: Option<&str>,
) -> Result<Json<Value>> {
    let keys: Option<&[&str]> = section.map(|section| match (view, section) {
        ("home", "overview") => &["invitations", "notifications", "jobs"][..],
        ("social", "friends") => &["friends"],
        ("social", "chat") => &["rooms"],
        ("social", "teams") => &["teams", "contribution_team_id"],
        ("social", "parties") => &["party"],
        ("social", "communities") => &["communities"],
        ("settings", "profile" | "privacy") => &[],
        ("settings", "linking") => &["links"],
        ("settings", "blocks") => &["blocks"],
        ("settings", "reports") => &["reports"],
        ("admin", "overview") => &["counts"],
        ("admin", "reports") => &["reports"],
        ("admin", "ranks") => &["ranks"],
        ("admin", "audit") => &["audit"],
        ("admin", "backups") => &["backups", "backup_policy"],
        ("life", "coins") => &["owners"],
        ("life", "land") => &["owners", "claims"],
        ("life", "homes") => &["homes"],
        ("life", "meetup") => &[],
        ("life", "achievements") => &["achievements"],
        ("life", "coin-history") => &["ledger"],
        ("market", "market") => &["listings", "owners", "claims"],
        ("market", "stored-assets") => &["assets", "owners", "claims"],
        ("market", "materials") => &["prices", "npc_remaining", "npc_reset"],
        ("expedition", "end") => &[
            "expeditions",
            "preparation",
            "cost",
            "duration_seconds",
            "destination",
            "lifetime",
            "access",
        ],
        _ => &["invalid"],
    });
    if keys.is_some_and(|keys| keys.contains(&"invalid")) {
        return Err(Error::missing());
    }
    let wants = |key: &str| keys.is_none_or(|keys| keys.contains(&key));
    let me = actor.id;
    let mut result = match view {
        "home" => {
            let invitations = if wants("invitations") {
                aggregate(&app,me,if section.is_some() { "SELECT coalesce(jsonb_agg(to_jsonb(v) ORDER BY v.created_at DESC),'[]') FROM (SELECT i.*,p.name AS sender_name FROM invitations i JOIN principals p ON p.id=i.sender WHERE i.recipient=$1 AND i.state='pending' AND i.expires_at>now() ORDER BY i.created_at DESC LIMIT 3) v" } else { "SELECT coalesce(jsonb_agg(to_jsonb(v) ORDER BY v.created_at DESC),'[]') FROM (SELECT i.*,p.name AS sender_name FROM invitations i JOIN principals p ON p.id=i.sender WHERE i.recipient=$1 AND i.state='pending' AND i.expires_at>now() ORDER BY i.created_at DESC LIMIT 100) v" }).await?
            } else {
                json!([])
            };
            let notifications = if wants("notifications") {
                aggregate(&app,me,if section.is_some() { "SELECT coalesce(jsonb_agg(to_jsonb(v) ORDER BY v.id DESC),'[]') FROM (SELECT * FROM notifications WHERE account_id=$1 AND (kind<>'job_finished' OR coalesce(body->>'kind','') NOT IN ('server.logs','server.files','server.file.read')) AND read_at IS NULL ORDER BY id DESC LIMIT 3) v" } else { "SELECT coalesce(jsonb_agg(to_jsonb(v) ORDER BY v.id DESC),'[]') FROM (SELECT * FROM notifications WHERE account_id=$1 AND (kind<>'job_finished' OR coalesce(body->>'kind','') NOT IN ('server.logs','server.files','server.file.read')) ORDER BY id DESC LIMIT 100) v" }).await?
            } else {
                json!([])
            };
            let jobs = if wants("jobs") {
                aggregate(&app,me,if section.is_some() { "SELECT coalesce(jsonb_agg(to_jsonb(v) ORDER BY v.created_at DESC),'[]') FROM (SELECT id,kind,state,progress,result,error,created_at FROM jobs WHERE actor=$1 AND kind NOT IN ('server.logs','server.files','server.file.read') ORDER BY created_at DESC LIMIT 3) v" } else { "SELECT coalesce(jsonb_agg(to_jsonb(v) ORDER BY v.created_at DESC),'[]') FROM (SELECT id,kind,state,progress,result,error,created_at FROM jobs WHERE actor=$1 AND kind NOT IN ('server.logs','server.files','server.file.read') ORDER BY created_at DESC LIMIT 100) v" }).await?
            } else {
                json!([])
            };
            json!({"invitations":invitations,"notifications":notifications,"jobs":jobs})
        }
        "play" => crate::player_views::play(&app, &actor).await?,
        "social" => {
            let friends = if wants("friends") {
                serde_json::to_value(crate::player_views::friends(&app, me).await?)
                    .map_err(Error::internal)?
            } else {
                json!([])
            };
            let rooms = if wants("rooms") {
                let mut db = app.db.acquire().await?;
                crate::timeline::rooms(&mut db, me, None).await?["rooms"].clone()
            } else {
                json!([])
            };
            let teams = if wants("teams") {
                teams::collection(&app, me).await?
            } else {
                json!({"teams":[],"contribution_team_id":null})
            };
            let party: Option<Value> = if wants("party") {
                sqlx::query_scalar("SELECT to_jsonb(p)||jsonb_build_object('name',r.name,'members',(SELECT coalesce(jsonb_agg(to_jsonb(pm)||jsonb_build_object('name',pp.name)),'[]') FROM party_members pm JOIN principals pp ON pp.id=pm.account_id WHERE pm.party_id=p.id)) FROM parties p JOIN party_members m ON m.party_id=p.id JOIN rooms r ON r.id=p.room_id WHERE m.account_id=$1 AND p.closed_at IS NULL").bind(me).fetch_optional(&app.db).await?
            } else {
                None
            };
            let communities = if wants("communities") {
                aggregate(&app,me,"SELECT coalesce(jsonb_agg(to_jsonb(c)),'[]') FROM communities c WHERE c.owner=$1 OR EXISTS(SELECT 1 FROM community_members m WHERE m.community_id=c.id AND m.account_id=$1)").await?
            } else {
                json!([])
            };
            json!({"friends":friends,"rooms":rooms,"teams":teams["teams"],"contribution_team_id":teams["contribution_team_id"],"party":party,"communities":communities})
        }
        "life" => {
            let owners = if wants("owners") {
                teams::owners(&app, me).await?
            } else {
                json!([])
            };
            let claims = if wants("claims") {
                teams::claims(&app, me).await?
            } else {
                json!([])
            };
            let homes = if wants("homes") {
                aggregate(&app,me,"SELECT coalesce(jsonb_agg(to_jsonb(h) ORDER BY h.name),'[]') FROM homes h JOIN profiles p ON p.id=h.profile_id WHERE p.account_id=$1 AND p.status='active'").await?
            } else {
                json!([])
            };
            let achievements = if wants("achievements") {
                teams::achievements(&app, me).await?
            } else {
                json!([])
            };
            let ledger = if wants("ledger") {
                aggregate(&app,me,"SELECT coalesce(jsonb_agg(to_jsonb(v) ORDER BY v.created_at DESC),'[]') FROM (SELECT l.id,l.kind,l.created_at,e.owner,e.amount,e.balance_after,l.detail FROM ledger l JOIN ledger_entries e ON e.transaction_id=l.id WHERE e.owner=$1 OR EXISTS(SELECT 1 FROM team_members m WHERE m.team_id=e.owner AND m.account_id=$1) ORDER BY l.created_at DESC LIMIT 100) v").await?
            } else {
                json!([])
            };
            json!({"owners":owners,"claims":claims,"homes":homes,"achievements":achievements,"ledger":ledger})
        }
        "market" => {
            let owners = if wants("owners") {
                teams::owners(&app, me).await?
            } else {
                json!([])
            };
            let claims = if wants("claims") {
                teams::claims(&app, me).await?
            } else {
                json!([])
            };
            let listings: Value = if wants("listings") {
                sqlx::query_scalar("SELECT coalesce(jsonb_agg(to_jsonb(v) ORDER BY v.created_at DESC),'[]') FROM (SELECT l.*,a.title,a.title_message,a.kind,coalesce(a.manifest->'summary',jsonb_build_object('dimensions',a.manifest->'dimensions','blocks',a.manifest->'blocks','materials',a.manifest->'materials','containers',a.manifest->'containers','entities',a.manifest->'entities','contents_included',a.manifest->'contents_included','location',CASE WHEN a.kind='land' THEN a.manifest->'source' ELSE NULL END)) AS manifest,p.name AS seller_name FROM listings l JOIN assets a ON a.id=l.asset_id JOIN principals p ON p.id=l.seller WHERE l.state='active' ORDER BY l.created_at DESC LIMIT 200) v").fetch_one(&app.db).await?
            } else {
                json!([])
            };
            let assets = if wants("assets") {
                aggregate(&app,me,"SELECT coalesce(jsonb_agg(to_jsonb(a) ORDER BY a.created_at DESC),'[]') FROM assets a WHERE a.owner=$1 OR EXISTS(SELECT 1 FROM team_members m WHERE m.team_id=a.owner AND m.account_id=$1) OR a.manifest->'required_consents' @> to_jsonb(ARRAY[$1::text])").await?
            } else {
                json!([])
            };
            let prices: Value = if wants("prices") {
                sqlx::query_scalar("SELECT coalesce(jsonb_agg(to_jsonb(p) ORDER BY material),'[]') FROM npc_prices p WHERE enabled").fetch_one(&app.db).await?
            } else {
                json!([])
            };
            let spent: i64 = if wants("npc_remaining") {
                sqlx::query_scalar("SELECT coalesce((SELECT d.coins FROM npc_daily d JOIN profiles p ON p.id=d.profile_id WHERE p.account_id=$1 AND p.status='active' AND d.day=(now() AT TIME ZONE 'UTC')::date),0)").bind(me).fetch_one(&app.db).await?
            } else {
                0
            };
            json!({"listings":listings,"assets":assets,"owners":owners,"claims":claims,"prices":prices,"npc_remaining":2000-spent,"npc_reset":"UTC 00:00","fee_percent":5})
        }
        "expedition" => {
            let mut db = app.db.acquire().await?;
            crate::expeditions::view(&mut db, me).await?
        }
        "servers" => {
            json!({"servers":crate::player_views::servers(&app, &actor, true).await?,"hosting":crate::hosting_projection::for_account(&app,me).await?})
        }
        "settings" => json!({
            "blocks":if wants("blocks") { aggregate(&app,me,"SELECT coalesce(jsonb_agg(jsonb_build_object('id',b.target,'name',p.name)),'[]') FROM blocks b JOIN principals p ON p.id=b.target WHERE b.actor=$1").await? } else { json!([]) },
            "reports":if wants("reports") { aggregate(&app,me,"SELECT coalesce(jsonb_agg(jsonb_build_object('id',id,'reason',reason,'status',status,'resolution',resolution,'created_at',created_at) ORDER BY created_at DESC),'[]') FROM reports WHERE reporter=$1").await? } else { json!([]) },
            "links":if wants("links") { aggregate(&app,me,"SELECT coalesce(jsonb_agg(jsonb_build_object('id',l.id,'initiator',l.initiator,'candidate',l.candidate,'state',l.state,'selected_profile',l.selected_profile,'expires_at',l.expires_at,'profiles',(SELECT coalesce(jsonb_agg(to_jsonb(p)||jsonb_build_object('name',o.name,'wallet',to_jsonb(w))),'[]') FROM profiles p JOIN principals o ON o.id=p.account_id JOIN wallets w ON w.owner=p.account_id WHERE p.account_id IN (l.initiator,l.candidate) AND p.status<>'archived'))),'[]') FROM link_requests l WHERE (l.initiator=$1 OR l.candidate=$1) AND (l.expires_at>now() OR l.state='migrating')").await? } else { json!([]) }
        }),
        "admin" => {
            if !actor.admin {
                return Err(Error::forbidden());
            }
            let counts: Value = if wants("counts") {
                sqlx::query_scalar("SELECT jsonb_build_object('reports',(SELECT count(*) FROM reports WHERE status IN ('open','investigating')),'operations',(SELECT count(*) FROM jobs WHERE kind NOT IN ('server.logs','server.files','server.file.read') AND state IN ('queued','waiting','leased')),'backups',(SELECT count(*) FROM backups WHERE kind='official' AND state='ready'))").fetch_one(&app.db).await?
            } else {
                json!({})
            };
            let reports: Value = if wants("reports") {
                sqlx::query_scalar("SELECT coalesce(jsonb_agg(jsonb_build_object('id',id,'reporter',reporter,'target',target,'status',status,'created_at',created_at) ORDER BY created_at),'[]') FROM reports WHERE status IN ('open','investigating')").fetch_one(&app.db).await?
            } else {
                json!([])
            };
            let ranks: Value = if wants("ranks") {
                sqlx::query_scalar(
                    "SELECT coalesce(jsonb_agg(to_jsonb(r) ORDER BY id),'[]') FROM trust_ranks r",
                )
                .fetch_one(&app.db)
                .await?
            } else {
                json!([])
            };
            let backups: Value = if wants("backups") {
                sqlx::query_scalar("SELECT coalesce(jsonb_agg(to_jsonb(v) ORDER BY v.created_at DESC),'[]') FROM (SELECT * FROM backups ORDER BY created_at DESC LIMIT 100) v").fetch_one(&app.db).await?
            } else {
                json!([])
            };
            let audit: Value = if wants("audit") {
                sqlx::query_scalar("SELECT coalesce(jsonb_agg(to_jsonb(v) ORDER BY v.id DESC),'[]') FROM (SELECT * FROM audit ORDER BY id DESC LIMIT 100) v").fetch_one(&app.db).await?
            } else {
                json!([])
            };
            let latest: Option<chrono::DateTime<chrono::Utc>> = if wants("backup_policy") {
                sqlx::query_scalar(
                    "SELECT max(completed_at) FROM backups WHERE kind='official' AND state='ready'",
                )
                .fetch_one(&app.db)
                .await?
            } else {
                None
            };
            json!({"counts":counts,"reports":reports,"ranks":ranks,"backups":backups,"audit":audit,"backup_policy":{"enabled":app.config.automatic_backups&&!app.config.development,"hour_utc":app.config.backup_hour_utc,"daily":7,"weekly":4,"last_completed_at":latest}})
        }
        _ => return Err(Error::missing()),
    };
    if let Some(keys) = keys {
        result
            .as_object_mut()
            .unwrap()
            .retain(|key, _| keys.contains(&key.as_str()));
    }
    crate::system_message::project_system_content(&mut result);
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct MessageQuery {
    before: Option<i64>,
    q: Option<String>,
}
pub async fn messages(
    State(app): State<App>,
    actor: Actor,
    Path(room): Path<Uuid>,
    Query(query): Query<MessageQuery>,
) -> Result<Json<Value>> {
    let mut db = app.db.acquire().await?;
    crate::timeline::room_access(&mut db, actor.id, room).await?;
    let q = query.q.unwrap_or_default();
    if q.len() > 200 {
        return Err(Error::invalid("text.the_search_term_is_too_long"));
    }
    let sql = format!(
        "SELECT coalesce(jsonb_agg(to_jsonb(v) ORDER BY v.id),'[]') FROM (SELECT m.id,m.room_id,m.author,p.name AS author_name,CASE WHEN m.deleted_at IS NULL THEN m.body ELSE '' END AS body,m.created_at,m.deleted_at FROM messages m JOIN principals p ON p.id=m.author WHERE m.room_id=$2 AND m.id<$3 AND {} AND ($4='' OR (m.deleted_at IS NULL AND position(lower($4) in lower(m.body))>0)) ORDER BY m.id DESC LIMIT 100) v",
        crate::timeline::AUTHOR_VISIBLE
    );
    let messages: Value = sqlx::query_scalar(&sql)
        .bind(actor.id)
        .bind(room)
        .bind(query.before.unwrap_or(i64::MAX))
        .bind(q)
        .fetch_one(&mut *db)
        .await?;
    Ok(Json(json!({"messages":messages})))
}
#[derive(Deserialize)]
pub struct PlayerQuery {
    q: String,
}
pub async fn players(
    State(app): State<App>,
    actor: Actor,
    Query(query): Query<PlayerQuery>,
) -> Result<Json<Value>> {
    if query.q.trim().is_empty() || query.q.len() > 128 {
        return Ok(Json(json!({"players":[]})));
    }
    let value:Value=sqlx::query_scalar("SELECT coalesce(jsonb_agg(to_jsonb(v)),'[]') FROM (SELECT a.id,p.name,r.name AS rank,r.name_message AS rank_message FROM accounts a JOIN principals p ON p.id=a.id JOIN trust_ranks r ON r.id=a.trust_rank WHERE a.merged_into IS NULL AND a.id<>$1 AND NOT EXISTS(SELECT 1 FROM blocks b WHERE (b.actor=$1 AND b.target=a.id) OR (b.actor=a.id AND b.target=$1)) AND (p.name ILIKE '%'||replace(replace(replace($2,'\\','\\\\'),'%','\\%'),'_','\\_')||'%' OR a.id::text=$2) ORDER BY p.name LIMIT 30) v").bind(actor.id).bind(query.q).fetch_one(&app.db).await?;
    Ok(Json(json!({"players":value})))
}
pub async fn job(
    State(app): State<App>,
    actor: Actor,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>> {
    crate::server_tools::read_job(State(app), actor, Path(id)).await
}
pub async fn evidence(db: &mut PgConnection, actor: Uuid, ids: &[i64]) -> Result<Value> {
    if ids.len() > 30 {
        return Err(Error::invalid("text.submit_up_to_30_messages"));
    }
    let distinct: std::collections::BTreeSet<_> = ids.iter().collect();
    if distinct.len() != ids.len() {
        return Err(Error::invalid(
            "text.the_selection_contains_duplicate_messages",
        ));
    }
    let readable:i64=sqlx::query_scalar("SELECT count(*) FROM messages m JOIN room_members r ON r.room_id=m.room_id AND r.account_id=$1 WHERE m.id=ANY($2) AND m.deleted_at IS NULL").bind(actor).bind(ids).fetch_one(&mut *db).await?;
    if readable != ids.len() as i64 {
        return Err(Error::forbidden());
    }
    // Exact submitted IDs only. No operator endpoint can fetch unreported private history.
    Ok(sqlx::query_scalar("SELECT coalesce(jsonb_agg(jsonb_build_object('id',m.id,'room_id',m.room_id,'author',m.author,'author_name',p.name,'body',m.body,'created_at',m.created_at) ORDER BY m.id),'[]') FROM messages m JOIN principals p ON p.id=m.author WHERE m.id=ANY($1)").bind(ids).fetch_one(db).await?)
}
#[derive(Deserialize)]
pub struct EvidenceRequest {
    message_ids: Vec<i64>,
}
pub async fn report_preview(
    State(app): State<App>,
    actor: Actor,
    Json(request): Json<EvidenceRequest>,
) -> Result<Json<Value>> {
    let mut db = app.db.acquire().await?;
    Ok(Json(
        json!({"evidence":evidence(&mut db,actor.id,&request.message_ids).await?}),
    ))
}
pub async fn report(
    State(app): State<App>,
    actor: Actor,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>> {
    let mut tx = app.db.begin().await?;
    let value: Value =
        sqlx::query_scalar("SELECT to_jsonb(r) FROM reports r WHERE id=$1 AND (reporter=$2 OR $3)")
            .bind(id)
            .bind(actor.id)
            .bind(actor.admin)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(Error::missing)?;
    if actor.admin {
        audit(&mut tx, actor.id, "report.read", id, json!({})).await?;
    }
    tx.commit().await?;
    Ok(Json(value))
}
