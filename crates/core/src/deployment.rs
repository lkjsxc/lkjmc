//! Quiesce new operations before the root GitOps manager changes runtimes.
//! Only root's reviewed database management path can close or reopen this gate.
use crate::error::{Error, Result};
use serde_json::Value;
use sqlx::PgConnection;

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
            "サーバーを更新しています。完了してからもう一度お試しください。",
        ));
    }
    Ok(())
}
