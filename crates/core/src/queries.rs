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

pub async fn ready(State(app): State<App>) -> Result<Json<Value>> {
    sqlx::query("SELECT 1").execute(&app.db).await?;
    Ok(Json(
        json!({"ready":true,"version":env!("CARGO_PKG_VERSION"),"login_configured":app.config.oidc_issuer.is_some(),"deployment_gate_supported":true}),
    ))
}
pub async fn me(State(app): State<App>, actor: Actor) -> Result<Json<Value>> {
    let account:Value=sqlx::query_scalar("SELECT jsonb_build_object('id',a.id,'name',p.name,'administrator',a.administrator,'language',a.language,'rank',to_jsonb(r),'dm_policy',a.dm_policy,'activity_policy',a.activity_policy,'profile',(SELECT to_jsonb(f) FROM profiles f WHERE f.account_id=a.id AND f.status<>'archived'),'identities',(SELECT coalesce(jsonb_agg(jsonb_build_object('issuer',i.issuer,'display_name',i.display_name)),'[]') FROM identities i WHERE i.account_id=a.id)) FROM accounts a JOIN principals p ON p.id=a.id JOIN trust_ranks r ON r.id=a.trust_rank WHERE a.id=$1").bind(actor.id).fetch_one(&app.db).await?;
    Ok(Json(
        json!({"account":account,"csrf":actor.csrf,"game_address":"lkjsxc.com:25591","voice_available":app.config.voice_url.is_some(),"development":app.config.development}),
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
        ("social", "teams") => &["team"],
        ("social", "parties") => &["party"],
        ("social", "communities") => &["communities"],
        ("settings", "profile" | "privacy") => &[],
        ("settings", "linking") => &["links"],
        ("settings", "blocks") => &["blocks"],
        ("settings", "reports") => &["reports"],
        ("admin", "overview") => &["counts"],
        ("admin", "reports") => &["reports"],
        ("admin", "ranks") => &["ranks"],
        ("admin", "jobs") => &["jobs"],
        ("admin", "audit") => &["audit"],
        ("admin", "backups") => &["backups", "backup_policy"],
        ("life", "coins") => &["owners"],
        ("life", "land") => &["owners", "claims"],
        ("life", "homes") => &["homes"],
        ("life", "meetup") => &[],
        ("life", "achievements") => &["achievements"],
        ("life", "coin-history") => &["ledger"],
        ("market", "market") => &["listings"],
        ("market", "stored-assets") => &["assets"],
        ("market", "materials") => &["prices", "npc_remaining", "npc_reset"],
        ("adventure", "end") => &["adventures", "cost", "duration_seconds"],
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
        "play" => {
            json!({"servers":aggregate(&app,me,"SELECT coalesce(jsonb_agg(to_jsonb(v) ORDER BY v.kind,v.name),'[]') FROM (SELECT s.id,s.name,s.kind,s.visibility,s.version,s.software,s.capabilities,s.desired,CASE WHEN s.last_observed_at<now()-interval '45 seconds' THEN 'unknown' ELSE s.observed END AS observed,s.players,s.error,s.last_observed_at,s.maintenance FROM servers s WHERE s.visibility='public' OR s.owner=$1 OR EXISTS(SELECT 1 FROM server_members m WHERE m.server_id=s.id AND m.account_id=$1) OR EXISTS(SELECT 1 FROM community_members m WHERE m.community_id=s.community_id AND m.account_id=$1)) v").await?})
        }
        "social" => {
            let friends = if wants("friends") {
                aggregate(&app,me,"SELECT coalesce(jsonb_agg(to_jsonb(v) ORDER BY v.name),'[]') FROM (SELECT a.id,p.name,f.state,f.requester,CASE WHEN a.activity_policy<>'none' AND f.state='accepted' AND g.lease_until>now() THEN g.server_id ELSE NULL END AS server_id FROM friendships f JOIN accounts a ON a.id=CASE WHEN f.first_id=$1 THEN f.second_id ELSE f.first_id END JOIN principals p ON p.id=a.id LEFT JOIN game_sessions g ON g.account_id=a.id WHERE $1 IN (f.first_id,f.second_id)) v").await?
            } else {
                json!([])
            };
            let rooms = if wants("rooms") {
                aggregate(&app,me,"SELECT coalesce(jsonb_agg(to_jsonb(v) ORDER BY v.name),'[]') FROM (SELECT r.id,r.kind,r.name,m.role,(SELECT count(*) FROM messages x WHERE x.room_id=r.id AND x.created_at>m.read_at AND x.author<>$1 AND NOT EXISTS(SELECT 1 FROM blocks b WHERE b.actor=$1 AND b.target=x.author)) AS unread,(SELECT coalesce(jsonb_agg(jsonb_build_object('id',p.id,'name',p.name,'role',mm.role)),'[]') FROM room_members mm JOIN principals p ON p.id=mm.account_id WHERE mm.room_id=r.id) AS members FROM rooms r JOIN room_members m ON m.room_id=r.id WHERE m.account_id=$1 AND r.archived_at IS NULL) v").await?
            } else {
                json!([])
            };
            let team: Option<Value> = if wants("team") {
                sqlx::query_scalar("SELECT to_jsonb(t)||jsonb_build_object('name',p.name,'permissions',to_jsonb(m),'members',(SELECT coalesce(jsonb_agg(to_jsonb(tm)||jsonb_build_object('name',pp.name)),'[]') FROM team_members tm JOIN principals pp ON pp.id=tm.account_id WHERE tm.team_id=t.id)) FROM teams t JOIN team_members m ON m.team_id=t.id JOIN principals p ON p.id=t.id WHERE m.account_id=$1 AND t.disbanded_at IS NULL").bind(me).fetch_optional(&app.db).await?
            } else {
                None
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
            json!({"friends":friends,"rooms":rooms,"team":team,"party":party,"communities":communities})
        }
        "life" => {
            let owners = if wants("owners") {
                aggregate(&app,me,"SELECT coalesce(jsonb_agg(to_jsonb(p)||jsonb_build_object('wallet',to_jsonb(w),'land',to_jsonb(l),'used_chunks',(SELECT coalesce(sum(chunks),0) FROM claims c WHERE c.owner=p.id AND c.state<>'released'))),'[]') FROM principals p JOIN wallets w ON w.owner=p.id JOIN land_allowances l ON l.owner=p.id WHERE p.id=$1 OR EXISTS(SELECT 1 FROM team_members m WHERE m.team_id=p.id AND m.account_id=$1)").await?
            } else {
                json!([])
            };
            let claims = if wants("claims") {
                aggregate(&app,me,"SELECT coalesce(jsonb_agg(to_jsonb(c) ORDER BY c.name),'[]') FROM claims c WHERE c.state<>'released' AND (c.owner=$1 OR EXISTS(SELECT 1 FROM team_members m WHERE m.team_id=c.owner AND m.account_id=$1))").await?
            } else {
                json!([])
            };
            let homes = if wants("homes") {
                aggregate(&app,me,"SELECT coalesce(jsonb_agg(to_jsonb(h) ORDER BY h.name),'[]') FROM homes h JOIN profiles p ON p.id=h.profile_id WHERE p.account_id=$1 AND p.status='active'").await?
            } else {
                json!([])
            };
            let achievements = if wants("achievements") {
                aggregate(&app,me,"SELECT coalesce(jsonb_agg(to_jsonb(a)||jsonb_build_object('progress',coalesce(p.progress,0),'earned_at',p.earned_at)),'[]') FROM achievements a LEFT JOIN achievement_progress p ON p.achievement=a.key AND p.owner=CASE WHEN a.team THEN (SELECT team_id FROM team_members WHERE account_id=$1) ELSE $1 END").await?
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
            let listings: Value = if wants("listings") {
                sqlx::query_scalar("SELECT coalesce(jsonb_agg(to_jsonb(v) ORDER BY v.created_at DESC),'[]') FROM (SELECT l.*,a.title,a.kind,coalesce(a.manifest->'summary',jsonb_build_object('dimensions',a.manifest->'dimensions','blocks',a.manifest->'blocks','materials',a.manifest->'materials','containers',a.manifest->'containers','entities',a.manifest->'entities','contents_included',a.manifest->'contents_included','location',CASE WHEN a.kind='land' THEN a.manifest->'source' ELSE NULL END)) AS manifest,p.name AS seller_name FROM listings l JOIN assets a ON a.id=l.asset_id JOIN principals p ON p.id=l.seller WHERE l.state='active' ORDER BY l.created_at DESC LIMIT 200) v").fetch_one(&app.db).await?
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
            json!({"listings":listings,"assets":assets,"prices":prices,"npc_remaining":2000-spent,"npc_reset":"UTC 00:00","fee_percent":5})
        }
        "adventure" => {
            json!({"adventures":aggregate(&app,me,"SELECT coalesce(jsonb_agg(to_jsonb(a)||jsonb_build_object('can_cancel',a.owner=$1 AND a.state IN ('preparing','activating'),'can_receive',EXISTS(SELECT 1 FROM assets s WHERE s.id=a.material_asset AND s.owner=$1 AND s.state='escrowed')) ORDER BY a.created_at DESC),'[]') FROM adventures a WHERE a.owner=$1 OR EXISTS(SELECT 1 FROM adventure_participants m WHERE m.adventure_id=a.id AND m.account_id=$1)").await?,"cost":{"coins":1000,"ender_eyes":12},"duration_seconds":10800})
        }
        "servers" => {
            let servers=aggregate(&app,me,"SELECT coalesce(jsonb_agg(to_jsonb(v) ORDER BY v.name,v.id),'[]') FROM (SELECT s.id,s.name,s.kind,s.software,s.version,s.memory_mib,s.cpu_millis,s.storage_mib,s.last_observed_at,CASE WHEN s.last_observed_at<now()-interval '45 seconds' THEN 'unknown' ELSE s.observed END AS observed FROM servers s WHERE s.owner=$1 OR EXISTS(SELECT 1 FROM server_members m WHERE m.server_id=s.id AND m.account_id=$1 AND m.role IN ('administrator','operator')) OR EXISTS(SELECT 1 FROM community_members m WHERE m.community_id=s.community_id AND m.account_id=$1 AND m.administrator) OR EXISTS(SELECT 1 FROM accounts WHERE id=$1 AND administrator)) v").await?;
            json!({"servers":servers})
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
                sqlx::query_scalar("SELECT jsonb_build_object('reports',(SELECT count(*) FROM reports WHERE status IN ('open','investigating')),'jobs',(SELECT count(*) FROM jobs WHERE kind NOT IN ('server.logs','server.files','server.file.read') AND state IN ('failed','waiting','leased')),'backups',(SELECT count(*) FROM backups WHERE kind='official' AND state='ready'))").fetch_one(&app.db).await?
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
            let jobs: Value = if wants("jobs") {
                sqlx::query_scalar("SELECT coalesce(jsonb_agg(to_jsonb(v) ORDER BY v.updated_at DESC),'[]') FROM (SELECT id,kind,server_id,state,error,progress,updated_at FROM jobs WHERE kind NOT IN ('server.logs','server.files','server.file.read') AND state IN ('failed','waiting','leased') ORDER BY updated_at DESC LIMIT 100) v").fetch_one(&app.db).await?
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
            json!({"counts":counts,"reports":reports,"ranks":ranks,"jobs":jobs,"backups":backups,"audit":audit,"backup_policy":{"enabled":app.config.automatic_backups&&!app.config.development,"hour_utc":app.config.backup_hour_utc,"daily":7,"weekly":4,"last_completed_at":latest}})
        }
        _ => return Err(Error::missing()),
    };
    if let Some(keys) = keys {
        result
            .as_object_mut()
            .unwrap()
            .retain(|key, _| keys.contains(&key.as_str()));
    }
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
    crate::social::room_member(&mut db, room, actor.id).await?;
    let q = query.q.unwrap_or_default();
    if q.len() > 200 {
        return Err(Error::invalid("The search term is too long."));
    }
    let messages:Value=sqlx::query_scalar("SELECT coalesce(jsonb_agg(to_jsonb(v) ORDER BY v.id),'[]') FROM (SELECT m.id,m.room_id,m.author,p.name AS author_name,m.body,m.created_at,m.deleted_at FROM messages m JOIN principals p ON p.id=m.author WHERE m.room_id=$1 AND m.id<$2 AND NOT EXISTS(SELECT 1 FROM blocks b WHERE b.actor=$3 AND b.target=m.author) AND ($4='' OR m.body ILIKE '%'||replace(replace(replace($4,'\\','\\\\'),'%','\\%'),'_','\\_')||'%') ORDER BY m.id DESC LIMIT 100) v")
        .bind(room).bind(query.before.unwrap_or(i64::MAX)).bind(actor.id).bind(q).fetch_one(&mut *db).await?;
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
    let value:Value=sqlx::query_scalar("SELECT coalesce(jsonb_agg(to_jsonb(v)),'[]') FROM (SELECT a.id,p.name,r.name AS rank FROM accounts a JOIN principals p ON p.id=a.id JOIN trust_ranks r ON r.id=a.trust_rank WHERE a.merged_into IS NULL AND a.id<>$1 AND NOT EXISTS(SELECT 1 FROM blocks b WHERE (b.actor=$1 AND b.target=a.id) OR (b.actor=a.id AND b.target=$1)) AND (p.name ILIKE '%'||replace(replace(replace($2,'\\','\\\\'),'%','\\%'),'_','\\_')||'%' OR a.id::text=$2) ORDER BY p.name LIMIT 30) v").bind(actor.id).bind(query.q).fetch_one(&app.db).await?;
    Ok(Json(json!({"players":value})))
}
pub async fn job(
    State(app): State<App>,
    actor: Actor,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>> {
    let value:Value=sqlx::query_scalar("SELECT jsonb_build_object('id',id,'kind',kind,'server_id',server_id,'state',state,'progress',progress,'result',result,'error',error,'updated_at',updated_at) FROM jobs WHERE id=$1 AND (actor=$2 OR $3)").bind(id).bind(actor.id).bind(actor.admin).fetch_optional(&app.db).await?.ok_or_else(Error::missing)?;
    Ok(Json(value))
}
pub async fn evidence(db: &mut PgConnection, actor: Uuid, ids: &[i64]) -> Result<Value> {
    if ids.len() > 30 {
        return Err(Error::invalid("Submit up to 30 messages."));
    }
    let distinct: std::collections::BTreeSet<_> = ids.iter().collect();
    if distinct.len() != ids.len() {
        return Err(Error::invalid("The selection contains duplicate messages."));
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
