use crate::{
    App,
    auth::Actor,
    error::{Error, Result},
};
use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::Row;
use uuid::Uuid;

#[derive(Serialize)]
struct VoiceClaims {
    iss: String,
    sub: String,
    name: String,
    nbf: i64,
    exp: i64,
    video: Value,
}
fn jwt(app: &App, actor: Uuid, name: String, room: Uuid, admin: bool) -> Result<String> {
    let key = app
        .config
        .voice_key
        .as_ref()
        .ok_or_else(|| Error::unavailable("音声サービスの接続設定を待っています。"))?;
    let secret = app
        .config
        .voice_secret
        .as_ref()
        .ok_or_else(|| Error::unavailable("音声サービスの接続設定を待っています。"))?;
    let video = if admin {
        json!({"roomAdmin":true,"room":format!("lkjmc-{room}")})
    } else {
        json!({"roomJoin":true,"room":format!("lkjmc-{room}"),"canPublish":true,"canSubscribe":true,"canPublishData":false,"canPublishSources":["microphone"]})
    };
    let now = chrono::Utc::now().timestamp();
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &VoiceClaims {
            iss: key.clone(),
            sub: actor.to_string(),
            name,
            nbf: now - 5,
            exp: now + 600,
            video,
        },
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(Error::internal)
}
pub async fn voice_token(
    State(app): State<App>,
    actor: Actor,
    Path(room): Path<Uuid>,
) -> Result<Json<Value>> {
    let mut tx = app.db.begin().await?;
    crate::social::room_member(&mut tx, room, actor.id).await?;
    let blocked:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM room_members m JOIN blocks b ON (b.actor=$1 AND b.target=m.account_id) OR (b.target=$1 AND b.actor=m.account_id) WHERE m.room_id=$2)").bind(actor.id).bind(room).fetch_one(&mut *tx).await?;
    if blocked {
        return Err(Error::conflict(
            "ブロック関係のあるメンバーがいるため、この音声ルームには参加できません。",
        ));
    }
    let url = app
        .config
        .voice_url
        .as_ref()
        .ok_or_else(|| Error::unavailable("音声サービスはまだ接続されていません。"))?;
    let name: String = sqlx::query_scalar("SELECT name FROM principals WHERE id=$1")
        .bind(actor.id)
        .fetch_one(&mut *tx)
        .await?;
    let token = jwt(&app, actor.id, name, room, false)?;
    sqlx::query("INSERT INTO voice_sessions(room_id,account_id,expires_at) VALUES($1,$2,now()+interval '24 hours') ON CONFLICT(room_id,account_id) DO UPDATE SET expires_at=now()+interval '24 hours'").bind(room).bind(actor.id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(json!({"url":url,"token":token,"recording":false})))
}
pub(super) async fn revoke(app: &App) -> Result<()> {
    let Some(url) = &app.config.voice_url else {
        return Ok(());
    };
    let revoked=sqlx::query("SELECT v.* FROM voice_sessions v WHERE v.expires_at<now() OR NOT EXISTS(SELECT 1 FROM room_members m JOIN rooms r ON r.id=m.room_id JOIN accounts a ON a.id=m.account_id WHERE m.room_id=v.room_id AND m.account_id=v.account_id AND r.archived_at IS NULL AND a.merged_into IS NULL AND (a.banned_until IS NULL OR a.banned_until<now())) OR EXISTS(SELECT 1 FROM room_members m JOIN blocks b ON (b.actor=v.account_id AND b.target=m.account_id) OR (b.target=v.account_id AND b.actor=m.account_id) WHERE m.room_id=v.room_id)").fetch_all(&app.db).await?;
    for row in revoked {
        let room: Uuid = row.get("room_id");
        let account: Uuid = row.get("account_id");
        let token = jwt(app, account, String::new(), room, true)?;
        let base = url
            .replacen("wss://", "https://", 1)
            .replacen("ws://", "http://", 1);
        let response = app
            .http
            .post(format!(
                "{}/twirp/livekit.RoomService/RemoveParticipant",
                base.trim_end_matches('/')
            ))
            .bearer_auth(token)
            .json(&json!({"room":format!("lkjmc-{room}"),"identity":account.to_string()}))
            .send()
            .await
            .map_err(Error::internal)?;
        if response.status().is_success() || response.status() == reqwest::StatusCode::NOT_FOUND {
            sqlx::query("DELETE FROM voice_sessions WHERE room_id=$1 AND account_id=$2")
                .bind(room)
                .bind(account)
                .execute(&app.db)
                .await?;
        }
    }
    Ok(())
}
