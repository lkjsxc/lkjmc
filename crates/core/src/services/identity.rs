use super::{Service, uuid};
use crate::{
    App,
    error::{Error, Result},
};
use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Request {
    lease_token: Uuid,
}

pub async fn status(
    State(app): State<App>,
    service: Service,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>> {
    service.require("official")?;
    let value:Value=sqlx::query_scalar("SELECT jsonb_build_object('state',state,'result',result) FROM jobs WHERE id=$1 AND kind='identity.migrate' AND server_id=$2")
        .bind(id).bind(service.server_id).fetch_optional(&app.db).await?.ok_or_else(Error::missing)?;
    Ok(Json(value))
}

pub async fn ready(
    State(app): State<App>,
    service: Service,
    Path(id): Path<Uuid>,
    Json(request): Json<Request>,
) -> Result<Json<Value>> {
    service.require("official")?;
    let mut tx = app.db.begin().await?;
    crate::economy::unpaused(&mut tx).await?;
    let payload:Value=sqlx::query_scalar("SELECT payload FROM jobs WHERE id=$1 AND kind='identity.migrate' AND server_id=$2 AND state='leased' AND lease_owner=$3 AND lease_token=$4 AND lease_until>now() FOR UPDATE")
        .bind(id).bind(service.server_id).bind(service.id).bind(request.lease_token).fetch_optional(&mut *tx).await?.ok_or_else(Error::forbidden)?;
    let accounts = [
        uuid(&payload, "retained_account")?,
        uuid(&payload, "other_account")?,
    ];
    // Once profiles move, admission and command paths deny new sessions and writes.
    // The proxy logs both connections out; Paper separately verifies its native players left.
    let ready:bool=sqlx::query_scalar("SELECT (SELECT count(*) FROM profiles WHERE account_id=ANY($1) AND status='moving')=2 AND NOT EXISTS(SELECT 1 FROM game_sessions WHERE account_id=ANY($1) AND lease_until>now()) AND NOT EXISTS(SELECT 1 FROM accounts WHERE id=ANY($1) AND combat_until>now())")
        .bind(&accounts).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(json!({"ready":ready})))
}
