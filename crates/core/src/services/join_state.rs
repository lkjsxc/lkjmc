//! Exact-session arrival reconciliation. Callers hold the account and job locks.
use crate::error::Result;
use serde_json::{Value, json};
use sqlx::{PgConnection, Row, postgres::PgRow};
use uuid::Uuid;

pub(super) async fn observed(
    db: &mut PgConnection,
    job: &PgRow,
    session: Option<&PgRow>,
) -> Result<bool> {
    let payload: Value = job.get("payload");
    let Some(session) = session else {
        return Ok(false);
    };
    if payload["session_id"] != json!(session.get::<Uuid, _>("session_id"))
        || payload["profile_id"] != json!(session.get::<Uuid, _>("profile_id"))
        || payload["native_uuid"] != json!(session.get::<Uuid, _>("native_uuid"))
        || job.get::<Uuid, _>("actor") != session.get::<Uuid, _>("account_id")
    {
        return Ok(false);
    }
    let Some(target) = job.get::<Option<Uuid>, _>("server_id") else {
        return Ok(false);
    };
    // A current destination is authoritative; fallback history must be from this
    // exact session/profile, after submission, and chronologically consistent.
    if session.get::<Option<Uuid>, _>("server_id") == Some(target) {
        return Ok(true);
    }
    Ok(sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM game_session_history h WHERE h.account_id=$1 AND h.session_id=$2 AND h.server_id=$3 AND h.profile_id=$4 AND h.last_seen_at >= $5 AND h.started_at <= h.last_seen_at AND h.last_seen_at <= now() AND EXISTS(SELECT 1 FROM game_session_history current WHERE current.account_id=h.account_id AND current.session_id=h.session_id AND current.profile_id=h.profile_id AND current.server_id=$6 AND current.last_seen_at >= h.last_seen_at))")
        .bind(session.get::<Uuid,_>("account_id")).bind(session.get::<Uuid,_>("session_id"))
        .bind(target).bind(session.get::<Uuid,_>("profile_id"))
        .bind(job.get::<chrono::DateTime<chrono::Utc>,_>("created_at"))
        .bind(session.get::<Option<Uuid>,_>("server_id"))
        .fetch_one(db).await?)
}

pub(super) async fn finish(
    db: &mut PgConnection,
    job: &PgRow,
    session: Option<&PgRow>,
    otherwise: &str,
    error: Option<&str>,
) -> Result<String> {
    let arrived = observed(db, job, session).await?;
    let state = if arrived { "succeeded" } else { otherwise };
    let payload: Value = job.get("payload");
    let mut result = json!({"effect":if arrived {"committed"} else {"none"},
        "session_id":payload["session_id"],
        "actual_server_id":session.and_then(|s| s.get::<Option<Uuid>,_>("server_id"))});
    if arrived {
        result["server_id"] = json!(job.get::<Option<Uuid>, _>("server_id"));
    }
    sqlx::query("UPDATE jobs SET state=$2,result=$3,error=$4,progress=progress||$5,lease_until=NULL,updated_at=now() WHERE id=$1")
        .bind(job.get::<Uuid,_>("id")).bind(state).bind(result)
        .bind(if arrived { None } else { error })
        .bind(json!({"phase":if arrived {"arrived"} else {state}})).execute(&mut *db).await?;
    crate::commands::notify(
        db,
        job.get("actor"),
        "job_finished",
        json!({"id":job.get::<Uuid,_>("id"),"kind":"player.join","state":state}),
    )
    .await?;
    Ok(state.to_owned())
}
