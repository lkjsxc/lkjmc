use crate::{
    App,
    auth::Actor,
    error::{Error, Result},
};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Default, Deserialize)]
pub struct PageQuery {
    pub section: Option<String>,
    pub cursor: Option<String>,
    #[serde(default)]
    pub unread: bool,
}
#[derive(Deserialize, Serialize)]
struct Cursor {
    at: chrono::DateTime<chrono::Utc>,
    id: Uuid,
}

pub async fn home(State(app): State<App>, actor: Actor) -> Result<Json<Value>> {
    let mut value =
        crate::queries::view_section(app.clone(), actor.clone(), "home", Some("overview"))
            .await?
            .0;
    for key in ["invitations", "notifications", "jobs"] {
        if let Some(rows) = value[key].as_array_mut() {
            rows.truncate(3);
        }
    }
    value["counts"] = sqlx::query_scalar::<_,Value>("SELECT jsonb_build_object('invitations',(SELECT count(*) FROM invitations WHERE recipient=$1 AND state='pending' AND expires_at>now()),'notifications',(SELECT count(*) FROM notifications WHERE account_id=$1 AND (kind<>'job_finished' OR coalesce(body->>'kind','') NOT IN ('server.logs','server.files','server.file.read')) AND read_at IS NULL),'jobs',(SELECT count(*) FROM jobs WHERE actor=$1 AND kind NOT IN ('server.logs','server.files','server.file.read') AND state IN ('queued','leased','waiting')))")
        .bind(actor.id).fetch_one(&app.db).await?;
    Ok(Json(value))
}

pub async fn history(
    State(app): State<App>,
    actor: Actor,
    Path(kind): Path<String>,
    Query(query): Query<PageQuery>,
) -> Result<Json<Value>> {
    let mut rows: Vec<Value>;
    if kind == "notifications" {
        let before = query
            .cursor
            .as_deref()
            .map(str::parse::<i64>)
            .transpose()
            .map_err(|_| Error::invalid("Invalid page cursor."))?
            .unwrap_or(i64::MAX);
        rows = sqlx::query_scalar("SELECT to_jsonb(n) FROM notifications n WHERE account_id=$1 AND (kind<>'job_finished' OR coalesce(body->>'kind','') NOT IN ('server.logs','server.files','server.file.read')) AND id<$2 AND (NOT $3 OR read_at IS NULL) ORDER BY id DESC LIMIT 26")
            .bind(actor.id).bind(before).bind(query.unread).fetch_all(&app.db).await?;
    } else {
        let before: Option<Cursor> = query
            .cursor
            .as_deref()
            .map(|s| {
                URL_SAFE_NO_PAD
                    .decode(s)
                    .ok()
                    .and_then(|b| serde_json::from_slice(&b).ok())
                    .ok_or_else(|| Error::invalid("Invalid page cursor."))
            })
            .transpose()?;
        let (at, id) = before.map(|c| (c.at, c.id)).unwrap_or((
            chrono::DateTime::<chrono::Utc>::MAX_UTC,
            Uuid::from_u128(u128::MAX),
        ));
        let sql = match kind.as_str() {
            "invitations" => {
                "SELECT to_jsonb(i)||jsonb_build_object('sender_name',p.name) FROM invitations i JOIN principals p ON p.id=i.sender WHERE recipient=$1 AND state='pending' AND expires_at>now() AND (i.created_at,i.id)<($2,$3) ORDER BY i.created_at DESC,i.id DESC LIMIT 26"
            }
            "activity" => {
                "SELECT jsonb_build_object('id',j.id,'kind',j.kind,'server_id',j.server_id,'state',j.state,'progress',j.progress,'error',j.error,'result',j.result,'created_at',j.created_at,'updated_at',j.updated_at) FROM jobs j WHERE actor=$1 AND kind NOT IN ('server.logs','server.files','server.file.read') AND (created_at,id)<($2,$3) ORDER BY created_at DESC,id DESC LIMIT 26"
            }
            _ => return Err(Error::missing()),
        };
        rows = sqlx::query_scalar(sql)
            .bind(actor.id)
            .bind(at)
            .bind(id)
            .fetch_all(&app.db)
            .await?;
    }
    let more = rows.len() > 25;
    rows.truncate(25);
    let cursor = if more {
        rows.last().map(|r| {
            if kind == "notifications" {
                r["id"].to_string()
            } else {
                URL_SAFE_NO_PAD.encode(
                    serde_json::to_vec(&Cursor {
                        at: serde_json::from_value(r["created_at"].clone()).unwrap(),
                        id: serde_json::from_value(r["id"].clone()).unwrap(),
                    })
                    .unwrap(),
                )
            }
        })
    } else {
        None
    };
    Ok(Json(
        json!({if kind=="activity" {"jobs"} else {kind.as_str()}:rows,"next_cursor":cursor}),
    ))
}

pub async fn server(
    State(app): State<App>,
    actor: Actor,
    Path(target): Path<String>,
    Query(query): Query<PageQuery>,
) -> Result<Json<Value>> {
    let id = if target == "official" {
        sqlx::query_scalar::<_,Uuid>("SELECT id FROM servers WHERE kind='official' AND (visibility='public' OR owner=$1 OR EXISTS(SELECT 1 FROM server_members WHERE server_id=servers.id AND account_id=$1) OR EXISTS(SELECT 1 FROM community_members WHERE community_id=servers.community_id AND account_id=$1) OR EXISTS(SELECT 1 FROM accounts WHERE id=$1 AND administrator)) ORDER BY created_at,id LIMIT 1").bind(actor.id).fetch_optional(&app.db).await?.ok_or_else(Error::missing)?
    } else {
        target.parse().map_err(|_| Error::missing())?
    };
    let section = query.section.as_deref().unwrap_or("overview");
    let managed = section.starts_with("manage-");
    if managed
        && ![
            "manage-overview",
            "manage-console",
            "manage-activity",
            "manage-files",
            "manage-backups",
            "manage-members",
            "manage-settings",
        ]
        .contains(&section)
    {
        return Err(Error::missing());
    }
    let mut db = app.db.acquire().await?;
    if managed {
        crate::hosting::server_permission(
            &mut db,
            actor.id,
            id,
            !matches!(
                section,
                "manage-overview" | "manage-console" | "manage-activity"
            ),
        )
        .await?;
    } else {
        crate::hosting::can_remain(&mut db, actor.id, id)
            .await
            .map_err(|_| Error::missing())?;
    }
    let mut s:Value=sqlx::query_scalar(if managed {"SELECT to_jsonb(s) FROM servers s WHERE id=$1"} else {"SELECT jsonb_build_object('id',id,'name',name,'kind',kind,'visibility',visibility,'software',software,'version',version,'observed',observed,'desired',desired,'players',players,'capabilities',capabilities,'maintenance',maintenance,'last_observed_at',last_observed_at) FROM servers WHERE id=$1"}).bind(id).fetch_optional(&mut *db).await?.ok_or_else(Error::missing)?;
    let administer = crate::hosting::server_permission(&mut db, actor.id, id, true)
        .await
        .is_ok();
    s["can_manage"] = json!(
        crate::hosting::server_permission(&mut db, actor.id, id, false)
            .await
            .is_ok()
    );
    s["can_administer"] = json!(administer);
    if s["last_observed_at"]
        .as_str()
        .and_then(|v| v.parse::<chrono::DateTime<chrono::Utc>>().ok())
        .is_some_and(|v| (chrono::Utc::now() - v).num_seconds() > 45)
    {
        s["observed"] = json!("unknown");
    }
    let mut value = json!({"server":s,"servers":[s]});
    if managed {
        let key_sql = match section {
            "manage-files" => Some((
                "artifacts",
                "SELECT to_jsonb(a) FROM artifacts a WHERE server_id=$1 ORDER BY created_at DESC",
            )),
            "manage-backups" => Some((
                "backups",
                "SELECT to_jsonb(b) FROM backups b WHERE server_id=$1 ORDER BY created_at DESC LIMIT 100",
            )),
            "manage-members" => Some((
                "members",
                "SELECT to_jsonb(m)||jsonb_build_object('name',p.name,'minecraft_operator_job',(SELECT jsonb_build_object('id',j.id,'state',j.state,'operator',j.payload->'operator','result',j.result,'error',j.error,'updated_at',j.updated_at) FROM jobs j WHERE j.server_id=m.server_id AND j.kind='server.operator' AND j.payload->>'member'=m.account_id::text ORDER BY j.created_at DESC,j.id DESC LIMIT 1)) FROM server_members m JOIN principals p ON p.id=m.account_id WHERE server_id=$1 ORDER BY p.name",
            )),
            _ => None,
        };
        if let Some((key, sql)) = key_sql {
            let rows: Vec<Value> = sqlx::query_scalar(sql).bind(id).fetch_all(&mut *db).await?;
            value["servers"][0][key] = json!(rows);
        }
        value["jobs"]=json!(sqlx::query_scalar::<_,Value>("SELECT jsonb_build_object('id',id,'kind',kind,'state',state,'progress',progress,'error',error,'created_at',created_at,'updated_at',updated_at) FROM jobs WHERE server_id=$1 AND kind NOT IN ('server.logs','server.files','server.file.read') ORDER BY created_at DESC,id DESC LIMIT 25").bind(id).fetch_all(&mut *db).await?);
    } else if section != "overview" {
        if value["server"]["kind"] != "official" {
            return Err(Error::missing());
        }
        let view = match section {
            "market" | "stored-assets" | "materials" => "market",
            "end" => "adventure",
            "land" | "homes" | "coins" | "coin-history" | "achievements" | "meetup" => "life",
            _ => return Err(Error::missing()),
        };
        let extra = crate::queries::view_section(app, actor, view, Some(section))
            .await?
            .0;
        value
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
    }
    Ok(Json(value))
}
