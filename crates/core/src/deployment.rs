//! Quiesce new operations before the root GitOps manager changes runtimes.
//! Only root's reviewed database management path can close or reopen this gate.
use crate::error::{Error, Result};
use serde_json::{Value, json};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

#[derive(Clone, clap::ValueEnum)]
pub enum Action {
    Inspect,
    Close,
    Open,
}

async fn activity(db: &mut PgConnection) -> Result<Value> {
    Ok(sqlx::query_scalar("SELECT jsonb_build_object('sessions',(SELECT count(*) FROM game_sessions WHERE lease_until>now()),'jobs',(SELECT count(*) FROM jobs WHERE state IN ('queued','leased','waiting')),'players',(SELECT coalesce(sum(players),0) FROM servers),'adventures',(SELECT count(*) FROM adventures WHERE state IN ('preparing','activating','active','closing','refunding')))")
        .fetch_one(db).await?)
}

// Invoked only by the local CLI using the existing private database authority.
// There is deliberately no HTTP route exposing this control to a host worker.
pub async fn control(pool: &PgPool, action: Action, owner: Option<Uuid>) -> Result<Value> {
    let mut tx = pool.begin().await?;
    // Complete even read-only and rejected operations before returning. Dropping
    // a transaction only queues its rollback; a subsequent controller could
    // otherwise observe the previous owner's advisory lock still held.
    match control_transaction(&mut tx, action, owner).await {
        Ok(result) => {
            tx.commit().await?;
            Ok(result)
        }
        Err(error) => {
            tx.rollback().await?;
            Err(error)
        }
    }
}

async fn control_transaction(
    db: &mut PgConnection,
    action: Action,
    owner: Option<Uuid>,
) -> Result<Value> {
    if !matches!(action, Action::Inspect) {
        let acquired: bool = sqlx::query_scalar(
            "SELECT pg_try_advisory_xact_lock(hashtextextended('lkjmc-deployment',0))",
        )
        .fetch_one(&mut *db)
        .await?;
        if !acquired {
            return Ok(json!({"status":"waiting-for-operations"}));
        }
    }
    // Read committed obtains a fresh snapshot after the exclusive lock above.
    // A single combined statement could inspect a snapshot from before a prior
    // operation committed, even after successfully acquiring the lock.
    let current: Option<Value> =
        sqlx::query_scalar("SELECT value FROM settings WHERE key='deployment_gate'")
            .fetch_optional(&mut *db)
            .await?;
    let activity = activity(&mut *db).await?;
    if matches!(action, Action::Inspect) {
        return Ok(json!({"status":"inspected","gate":current,"activity":activity}));
    }
    let owner = owner.ok_or_else(|| Error::invalid("A deployment plan owner ID is required."))?;
    if let Some(current) = &current {
        if current["closed"].as_bool().is_none()
            || (current["closed"] == true && current["owner"] != json!(owner))
        {
            return Err(Error::conflict(
                "Another deployment plan owns the maintenance gate.",
            ));
        }
    }
    let closing = matches!(action, Action::Close);
    if closing
        && activity
            .as_object()
            .unwrap()
            .values()
            .any(|n| n.as_i64() != Some(0))
    {
        return Ok(json!({"status":"waiting-for-idle","gate":current,"activity":activity}));
    }
    if !closing && current.as_ref().is_none_or(|v| v["owner"] != json!(owner)) {
        return Err(Error::conflict(
            "The deployment plan does not match the request to reopen access.",
        ));
    }
    if let Some(current) = &current {
        if current["closed"] == closing && current["owner"] == json!(owner) {
            return Ok(
                json!({"status":if closing {"closed"} else {"open"},"gate":current,"activity":activity}),
            );
        }
    }
    let gate = json!({"closed":closing,"owner":owner,"changed_at":chrono::Utc::now()});
    sqlx::query("INSERT INTO settings(key,value) VALUES('deployment_gate',$1) ON CONFLICT(key) DO UPDATE SET value=EXCLUDED.value")
        .bind(&gate).execute(&mut *db).await?;
    Ok(json!({"status":if closing {"closed"} else {"open"},"gate":gate,"activity":activity}))
}

pub async fn permits(db: &mut PgConnection) -> Result<bool> {
    // Hold until the caller's transaction commits. The manager takes the
    // exclusive form before checking for idle sessions/jobs and closing.
    sqlx::query("SELECT pg_advisory_xact_lock_shared(hashtextextended('lkjmc-deployment',0))")
        .execute(&mut *db)
        .await?;
    let gate: Option<Value> =
        sqlx::query_scalar("SELECT value FROM settings WHERE key='deployment_gate'")
            .fetch_optional(db)
            .await?;
    Ok(gate.is_none_or(|value| value["closed"].as_bool() == Some(false)))
}

pub async fn enter(db: &mut PgConnection) -> Result<()> {
    if !permits(db).await? {
        return Err(Error::unavailable(
            "The server is being updated. Please try again after the update.",
        ));
    }
    Ok(())
}
