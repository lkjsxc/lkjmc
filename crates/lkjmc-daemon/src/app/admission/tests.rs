use std::time::Duration;

use super::*;

#[tokio::test]
async fn shutdown_waits_for_inflight_admission() -> Result<(), String> {
    let admission = Admission::with_deadline(Duration::from_secs(1));
    let request = admission.try_admit().ok_or("admission missing")?;
    let worker = tokio::spawn(async move {
        request
            .run_blocking(|| std::thread::sleep(Duration::from_millis(50)))
            .await
    });
    tokio::time::sleep(Duration::from_millis(5)).await;
    admission.close();
    assert!(admission.try_admit().is_none());
    assert!(
        tokio::time::timeout(Duration::from_millis(5), admission.wait_for_idle())
            .await
            .is_err()
    );
    worker
        .await
        .map_err(|error| error.to_string())?
        .map_err(|_| "worker did not complete".to_string())?;
    admission
        .wait_for_idle()
        .await
        .map_err(|_| "shutdown join failed".to_string())?;
    assert_eq!(admission.observed_workers(), 1);
    Ok(())
}

#[tokio::test]
async fn completed_worker_is_observed_before_removal() -> Result<(), String> {
    let admission = Admission::with_deadline(Duration::from_secs(1));
    let request = admission.try_admit().ok_or("admission missing")?;
    request
        .run_blocking(|| ())
        .await
        .map_err(|_| "worker did not complete".to_string())?;
    drop(request);
    admission
        .wait_for_idle()
        .await
        .map_err(|_| "completed worker join failed".to_string())?;
    assert_eq!(admission.tracked_workers(), 0);
    assert_eq!(admission.observed_workers(), 1);
    Ok(())
}

#[tokio::test]
async fn outer_cancellation_keeps_worker_tracked_until_cleanup() -> Result<(), String> {
    let admission = Admission::with_deadline(Duration::from_secs(1));
    let request = admission.try_admit().ok_or("admission missing")?;
    let (started, observed) = tokio::sync::oneshot::channel();
    let outer = tokio::spawn(async move {
        request
            .run_blocking(move || {
                let _ = started.send(());
                std::thread::sleep(Duration::from_millis(50));
            })
            .await
    });
    observed.await.map_err(|error| error.to_string())?;
    outer.abort();
    let _ = outer.await;
    assert!(admission.tracked_workers() > 0);
    assert!(
        tokio::time::timeout(Duration::from_millis(5), admission.wait_for_idle())
            .await
            .is_err()
    );
    tokio::time::timeout(Duration::from_secs(1), admission.wait_for_idle())
        .await
        .map_err(|_| "worker cleanup exceeded its bound".to_string())?
        .map_err(|_| "cancelled worker join failed".to_string())?;
    assert_eq!(admission.tracked_workers(), 0);
    assert_eq!(admission.observed_workers(), 1);
    Ok(())
}

#[tokio::test]
async fn deadline_keeps_worker_tracked_until_cleanup() -> Result<(), String> {
    let admission = Admission::with_deadline(Duration::from_millis(10));
    let request = admission.try_admit().ok_or("admission missing")?;
    let (release, blocked) = std::sync::mpsc::channel();
    let result = request
        .run_blocking(move || blocked.recv_timeout(Duration::from_secs(1)))
        .await;
    assert!(matches!(result, Err(BlockingError::Deadline)));
    drop(request);
    assert!(admission.tracked_workers() > 0);
    release.send(()).map_err(|error| error.to_string())?;
    tokio::time::timeout(Duration::from_secs(1), admission.wait_for_idle())
        .await
        .map_err(|_| "timed-out worker was not joined".to_string())?
        .map_err(|_| "timed-out worker join failed".to_string())?;
    assert_eq!(admission.tracked_workers(), 0);
    assert_eq!(admission.observed_workers(), 1);
    Ok(())
}

#[test]
fn auth_budget_leaves_only_remaining_sql_time() -> Result<(), String> {
    let Ok(url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let database = crate::test_database::migrate(&url)?;
    let state = crate::app::AppState::with_config_path(
        Some(database.url().to_string()),
        1,
        "/tmp/lkjmc-config".to_string(),
        "/tmp/lkjmc-logs".to_string(),
        "/tmp/lkjmc-jars".to_string(),
        "/tmp/lkjmc-data".to_string(),
        None,
        None,
        None,
    );
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    runtime.block_on(async {
        let admission = Admission::with_deadline(Duration::from_millis(400));
        let request = admission.try_admit().ok_or("admission missing")?;
        request
            .run_blocking(|| std::thread::sleep(Duration::from_millis(250)))
            .await
            .map_err(|_| "authentication budget worker failed".to_string())?;
        let query_state = state.clone();
        let result = request
            .run_blocking(move || {
                let mut client = query_state.request_database_connection()?;
                client
                    .batch_execute("/* admission-budget-probe */ select pg_sleep(1)")
                    .map_err(lkjmc_store::error::StoreError::from)
            })
            .await;
        match result {
            Err(BlockingError::Deadline) => {}
            Ok(Err(error)) if error.is_deadline() => {}
            _ => return Err("handler SQL did not consume only the remaining budget".to_string()),
        }
        drop(request);
        tokio::time::timeout(Duration::from_secs(1), admission.wait_for_idle())
            .await
            .map_err(|_| "deadline worker was not joined".to_string())?
            .map_err(|_| "deadline worker join failed".to_string())
    })?;
    let mut inspect =
        lkjmc_store::pool::connect(database.url()).map_err(|error| error.to_string())?;
    let active: i64 = inspect
        .query_one(
            "select count(*) from pg_stat_activity where state = 'active' and pid <> pg_backend_pid() and query like '%admission-budget-probe%'",
            &[],
        )
        .map_err(|error| error.to_string())?
        .get(0);
    assert_eq!(active, 0);
    Ok(())
}
