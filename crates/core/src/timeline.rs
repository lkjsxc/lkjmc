//! Private timeline reads use current membership, never administrator privilege.
use crate::{
    App,
    auth::Actor,
    error::{Error, Result},
};
use axum::{
    Json,
    extract::{Query, RawQuery, State},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::PgConnection;
use std::collections::BTreeSet;
use uuid::Uuid;

// Shared by room lists and message reads. A blocked DM is not a selectable target.
pub const ROOM_VISIBLE: &str = "r.archived_at IS NULL AND EXISTS(SELECT 1 FROM room_members membership WHERE membership.room_id=r.id AND membership.account_id=$1) AND (r.kind<>'dm' OR NOT EXISTS(SELECT 1 FROM room_members other JOIN blocks b ON (b.actor=$1 AND b.target=other.account_id) OR (b.target=$1 AND b.actor=other.account_id) WHERE other.room_id=r.id AND other.account_id<>$1))";
pub const AUTHOR_VISIBLE: &str = "NOT EXISTS(SELECT 1 FROM blocks b WHERE (b.actor=$1 AND b.target=m.author) OR (b.target=$1 AND b.actor=m.author))";

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimelineQuery {
    kind: Option<String>,
    room: Option<Uuid>,
    before: Option<String>,
    known: Option<String>,
    rooms_before: Option<Uuid>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Cursor {
    actor: Uuid,
    kind: String,
    room: Option<Uuid>,
    at: chrono::DateTime<chrono::Utc>,
    id: String,
}
fn valid_id(id: &str) -> bool {
    match id.split_once(':') {
        Some(("message" | "notification", n)) => {
            n.parse::<i64>().is_ok_and(|v| v > 0 && v.to_string() == n)
        }
        Some(("job", n)) => n.parse::<Uuid>().is_ok_and(|v| v.to_string() == n),
        _ => false,
    }
}
fn known_ids(value: Option<&str>) -> Result<Vec<String>> {
    let Some(value) = value else {
        return Ok(vec![]);
    };
    let ids: Vec<_> = value.split(',').map(str::to_owned).collect();
    if ids.len() > 200
        || ids.iter().any(|id| !valid_id(id))
        || ids.iter().collect::<BTreeSet<_>>().len() != ids.len()
    {
        return Err(Error::invalid("Supply up to 200 distinct timeline IDs."));
    }
    Ok(ids)
}
pub async fn room_access(db: &mut PgConnection, actor: Uuid, room: Uuid) -> Result<()> {
    let visible: bool = sqlx::query_scalar(&format!(
        "SELECT EXISTS(SELECT 1 FROM rooms r WHERE r.id=$2 AND {ROOM_VISIBLE})"
    ))
    .bind(actor)
    .bind(room)
    .fetch_one(db)
    .await?;
    if !visible {
        return Err(Error::forbidden());
    }
    Ok(())
}
pub async fn rooms(db: &mut PgConnection, actor: Uuid, before: Option<Uuid>) -> Result<Value> {
    room_rows(db, actor, before, None).await
}
async fn room_rows(
    db: &mut PgConnection,
    actor: Uuid,
    before: Option<Uuid>,
    selected: Option<Uuid>,
) -> Result<Value> {
    let sql = format!(
        "SELECT jsonb_build_object('id',r.id,'kind',r.kind,'name',r.name,'role',member.role,'unread',(SELECT count(*) FROM messages m WHERE m.room_id=r.id AND m.created_at>member.read_at AND m.author<>$1 AND m.deleted_at IS NULL AND {AUTHOR_VISIBLE}),'members',(SELECT coalesce(jsonb_agg(to_jsonb(v) ORDER BY v.id),'[]') FROM (SELECT p.id,p.name,mm.role FROM room_members mm JOIN principals p ON p.id=mm.account_id WHERE mm.room_id=r.id ORDER BY p.id LIMIT 32) v),'member_count',(SELECT count(*) FROM room_members mm WHERE mm.room_id=r.id)) FROM rooms r JOIN room_members member ON member.room_id=r.id AND member.account_id=$1 WHERE {ROOM_VISIBLE} AND ($2::uuid IS NULL OR r.id>$2) AND ($3::uuid IS NULL OR r.id=$3) ORDER BY r.id LIMIT 101"
    );
    let mut rows: Vec<Value> = sqlx::query_scalar(&sql)
        .bind(actor)
        .bind(before)
        .bind(selected)
        .fetch_all(db)
        .await?;
    let more = rows.len() > 100;
    rows.truncate(100);
    let cursor = if more {
        rows.last().map(|r| r["id"].clone())
    } else {
        None
    };
    Ok(json!({"rooms":rows,"rooms_next_cursor":cursor}))
}
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoomQuery {
    before: Option<Uuid>,
    selected: Option<Uuid>,
    known: Option<String>,
}
pub async fn room_list(
    State(app): State<App>,
    actor: Actor,
    RawQuery(raw): RawQuery,
    Query(query): Query<RoomQuery>,
) -> Result<Json<Value>> {
    if raw.as_ref().is_some_and(|v| v.len() > 12 * 1024) {
        return Err(Error::invalid("The conversation query is too large."));
    }
    let known: Vec<Uuid> = query
        .known
        .as_deref()
        .map(|s| s.split(',').map(str::parse).collect())
        .transpose()
        .map_err(|_| Error::invalid("Invalid conversation IDs."))?
        .unwrap_or_default();
    if known.len() > 200 || known.iter().collect::<BTreeSet<_>>().len() != known.len() {
        return Err(Error::invalid("Supply up to 200 conversation IDs."));
    }
    let mut tx = app.db.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .execute(&mut *tx)
        .await?;
    let mut data = rooms(&mut tx, actor.id, query.before).await?;
    if let Some(selected) = query.selected {
        let chosen = room_rows(&mut tx, actor.id, None, Some(selected)).await?;
        if let Some(room) = chosen["rooms"].as_array().and_then(|r| r.first()) {
            if !data["rooms"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["id"] == room["id"])
            {
                data["rooms"].as_array_mut().unwrap().push(room.clone());
            }
        }
    }
    let visible: Vec<Uuid> = sqlx::query_scalar(&format!(
        "SELECT r.id FROM rooms r WHERE r.id=ANY($2) AND {ROOM_VISIBLE}"
    ))
    .bind(actor.id)
    .bind(&known)
    .fetch_all(&mut *tx)
    .await?;
    data["removed_room_ids"] = json!(
        known
            .iter()
            .filter(|id| !visible.contains(id))
            .collect::<Vec<_>>()
    );
    tx.commit().await?;
    Ok(Json(data))
}

pub async fn read(
    State(app): State<App>,
    actor: Actor,
    RawQuery(raw): RawQuery,
    Query(query): Query<TimelineQuery>,
) -> Result<Json<Value>> {
    if raw.as_ref().is_some_and(|v| v.len() > 12 * 1024) {
        return Err(Error::invalid("The timeline query is too large."));
    }
    let kind = query.kind.as_deref().unwrap_or("all");
    if !matches!(kind, "all" | "messages" | "events") {
        return Err(Error::invalid("Choose all, messages or events."));
    }
    let known = known_ids(query.known.as_deref())?;
    let cursor: Option<Cursor> = query
        .before
        .as_deref()
        .map(|s| {
            if s.len() > 512 {
                return Err(Error::invalid("Invalid timeline cursor."));
            }
            let c: Cursor = URL_SAFE_NO_PAD
                .decode(s)
                .ok()
                .and_then(|b| serde_json::from_slice(&b).ok())
                .ok_or_else(|| Error::invalid("Invalid timeline cursor."))?;
            if c.actor != actor.id || c.room != query.room || c.kind != kind || !valid_id(&c.id) {
                return Err(Error::invalid(
                    "The timeline cursor belongs to a different view.",
                ));
            }
            Ok(c)
        })
        .transpose()?;
    let mut tx = app.db.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .execute(&mut *tx)
        .await?;
    if let Some(room) = query.room {
        room_access(&mut tx, actor.id, room).await?;
    }
    let sql = format!(
        r#"
WITH readable AS NOT MATERIALIZED (
 SELECT 'message:'||m.id AS id,m.created_at,
 jsonb_build_object('id','message:'||m.id,'type','message','message_id',m.id,'created_at',m.created_at,'room_id',r.id,'room_name',r.name,'room_kind',r.kind,'author',m.author,'author_name',p.name,'body',CASE WHEN m.deleted_at IS NULL THEN m.body ELSE '' END,'deleted_at',m.deleted_at) AS data
 FROM messages m JOIN rooms r ON r.id=m.room_id JOIN principals p ON p.id=m.author
 WHERE $3<>'events' AND ($2::uuid IS NULL OR r.id=$2) AND {ROOM_VISIBLE} AND {AUTHOR_VISIBLE}
 UNION ALL
 SELECT 'job:'||j.id,j.created_at,
 jsonb_build_object('id','job:'||j.id,'type','job','job_id',j.id,'created_at',j.created_at,'updated_at',j.updated_at,'kind',j.kind,'open',CASE WHEN j.kind='server.inspection' THEN j.payload->'open' ELSE NULL END,'state',j.state,'server_id',j.server_id,'server_name',s.name,'progress',jsonb_build_object('phase',j.progress->'phase','message',left(j.progress->>'message',2000)),'error',left(j.error,2000),'result',jsonb_build_object('effect',j.result->'effect','actual_server_id',j.result->'actual_server_id'))
 FROM jobs j LEFT JOIN servers s ON s.id=j.server_id
 WHERE j.actor=$1 AND $2::uuid IS NULL AND $3<>'messages' AND j.kind NOT IN ('server.logs','server.files','server.file.read') AND coalesce(j.payload->>'automatic','false')<>'true'
 UNION ALL
 SELECT 'notification:'||n.id,n.created_at,
 jsonb_build_object('id','notification:'||n.id,'notification_id',n.id,'type','notification','created_at',n.created_at,'kind',n.kind,'read_at',n.read_at,'body',CASE WHEN octet_length(n.body::text)<=8192 THEN n.body ELSE jsonb_build_object('detail_available',true) END)
 FROM notifications n WHERE n.account_id=$1 AND $2::uuid IS NULL AND $3<>'messages' AND n.kind NOT IN ('message','job_finished')
), page AS (
 SELECT * FROM readable WHERE $4::timestamptz IS NULL OR (created_at,id COLLATE "C")<($4,$5 COLLATE "C") ORDER BY created_at DESC,id COLLATE "C" DESC LIMIT 51
)
SELECT jsonb_build_object('page',(SELECT coalesce(jsonb_agg(data ORDER BY created_at DESC,id COLLATE "C" DESC),'[]') FROM page),'updates',(SELECT coalesce(jsonb_agg(data ORDER BY created_at,id COLLATE "C"),'[]') FROM readable WHERE id=ANY($6)))
"#
    );
    let data: Value = sqlx::query_scalar(&sql)
        .bind(actor.id)
        .bind(query.room)
        .bind(kind)
        .bind(cursor.as_ref().map(|c| c.at))
        .bind(cursor.as_ref().map(|c| &c.id))
        .bind(&known)
        .fetch_one(&mut *tx)
        .await?;
    let mut items = data["page"].as_array().cloned().unwrap_or_default();
    let more = items.len() > 50;
    items.truncate(50);
    let next = if more {
        items
            .last()
            .map(|item| -> Result<String> {
                Ok(URL_SAFE_NO_PAD.encode(
                    serde_json::to_vec(&Cursor {
                        actor: actor.id,
                        kind: kind.to_owned(),
                        room: query.room,
                        at: serde_json::from_value(item["created_at"].clone())
                            .map_err(Error::internal)?,
                        id: item["id"].as_str().unwrap().to_owned(),
                    })
                    .map_err(Error::internal)?,
                ))
            })
            .transpose()?
    } else {
        None
    };
    items.reverse();
    let mut updates = data["updates"].as_array().cloned().unwrap_or_default();
    for item in items.iter_mut().chain(updates.iter_mut()) {
        item["before_cursor"] = json!(
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&Cursor {
                    actor: actor.id,
                    kind: kind.to_owned(),
                    room: query.room,
                    at: serde_json::from_value(item["created_at"].clone())
                        .map_err(Error::internal)?,
                    id: item["id"].as_str().unwrap().to_owned()
                })
                .map_err(Error::internal)?
            )
        );
    }
    let present: BTreeSet<_> = updates.iter().filter_map(|v| v["id"].as_str()).collect();
    let removed: Vec<_> = known
        .iter()
        .filter(|id| !present.contains(id.as_str()))
        .collect();
    let mut response = rooms(&mut tx, actor.id, query.rooms_before).await?;
    response["items"] = json!(items);
    response["updates"] = json!(updates);
    response["removed_ids"] = json!(removed);
    response["next_cursor"] = json!(next);
    tx.commit().await?;
    if serde_json::to_vec(&response)
        .map_err(Error::internal)?
        .len()
        > 8 * 1024 * 1024
    {
        return Err(Error::unavailable(
            "This timeline page exceeds the response limit.",
        ));
    }
    Ok(Json(response))
}
