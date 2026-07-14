use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::oneshot;
use tokio::task::JoinHandle;

const INTERVAL: Duration = Duration::from_secs(60 * 60);
const DATABASE_BUDGET: Duration = Duration::from_secs(8);
type Action = Arc<dyn Fn() -> Result<lkjmc_store::sync::RetentionResult, String> + Send + Sync>;

#[derive(Clone, Default)]
pub(crate) struct Maintenance {
    inner: Arc<Inner>,
}

#[derive(Default)]
struct Inner {
    running: AtomicBool,
    starts: AtomicU64,
    runs: AtomicU64,
    last_successful_run: AtomicU64,
    archived: AtomicU64,
    deleted: AtomicU64,
    last_error: Mutex<Option<String>>,
    worker: Mutex<Option<Worker>>,
}

struct Worker {
    stop: oneshot::Sender<()>,
    task: JoinHandle<()>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Diagnostics {
    pub running: bool,
    pub singleton_count: u64,
    pub completed_runs: u64,
    pub last_successful_run: Option<u64>,
    pub archived_rows: u64,
    pub deleted_rows: u64,
    pub last_error: Option<String>,
}

impl Maintenance {
    pub(crate) fn start(&self, pool: Option<lkjmc_store::pool::Pool>) -> Result<(), String> {
        let Some(pool) = pool else {
            return Ok(());
        };
        let action = Arc::new(move || {
            let mut client = pool
                .get_timeout(DATABASE_BUDGET)
                .map_err(|_| "database-unavailable".to_string())?;
            lkjmc_store::pool::set_deadlines(&mut client, DATABASE_BUDGET)
                .map_err(|_| "database-deadline".to_string())?;
            let result = lkjmc_store::sync::run_retention(&mut client)
                .map_err(|_| "sync-retention-failed".to_string())?;
            lkjmc_store::observability::retain(&mut *client)
                .map_err(|_| "observability-retention-failed".to_string())?;
            Ok(result)
        });
        self.start_action(action, INTERVAL)
    }

    fn start_action(&self, action: Action, interval: Duration) -> Result<(), String> {
        let mut owner = self
            .inner
            .worker
            .lock()
            .map_err(|_| "maintenance lock poisoned")?;
        if owner.is_some() {
            return Err("sync maintenance worker already started".to_string());
        }
        let (stop, mut stopped) = oneshot::channel();
        let inner = Arc::clone(&self.inner);
        inner.starts.fetch_add(1, Ordering::AcqRel);
        inner.running.store(true, Ordering::Release);
        let task = tokio::spawn(async move {
            loop {
                let run = Arc::clone(&action);
                let result = tokio::task::spawn_blocking(move || run()).await;
                record(&inner, result);
                tokio::select! {
                    _ = &mut stopped => break,
                    _ = tokio::time::sleep(interval) => {}
                }
            }
            inner.running.store(false, Ordering::Release);
        });
        *owner = Some(Worker { stop, task });
        Ok(())
    }

    pub(crate) async fn shutdown(&self) -> Result<(), String> {
        let worker = self
            .inner
            .worker
            .lock()
            .map_err(|_| "maintenance lock poisoned")?
            .take();
        if let Some(worker) = worker {
            let _ = worker.stop.send(());
            worker
                .task
                .await
                .map_err(|error| format!("maintenance worker join: {error}"))?;
        }
        Ok(())
    }

    pub(crate) fn diagnostics(&self) -> Diagnostics {
        let success = self.inner.last_successful_run.load(Ordering::Acquire);
        Diagnostics {
            running: self.inner.running.load(Ordering::Acquire),
            singleton_count: self.inner.starts.load(Ordering::Acquire),
            completed_runs: self.inner.runs.load(Ordering::Acquire),
            last_successful_run: (success > 0).then_some(success),
            archived_rows: self.inner.archived.load(Ordering::Acquire),
            deleted_rows: self.inner.deleted.load(Ordering::Acquire),
            last_error: self
                .inner
                .last_error
                .lock()
                .ok()
                .and_then(|value| value.clone()),
        }
    }
}

fn record(
    inner: &Inner,
    result: Result<Result<lkjmc_store::sync::RetentionResult, String>, tokio::task::JoinError>,
) {
    let run = inner.runs.fetch_add(1, Ordering::AcqRel) + 1;
    match result {
        Ok(Ok(value)) => {
            inner.last_successful_run.store(run, Ordering::Release);
            inner.archived.fetch_add(value.archived, Ordering::AcqRel);
            inner.deleted.fetch_add(value.deleted, Ordering::AcqRel);
            if let Ok(mut error) = inner.last_error.lock() {
                *error = None;
            }
        }
        Ok(Err(error)) => set_error(inner, error),
        Err(_) => set_error(inner, "worker-join-failed".to_string()),
    }
}

fn set_error(inner: &Inner, message: String) {
    if let Ok(mut error) = inner.last_error.lock() {
        *error = Some(message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn singleton_starts_runs_and_joins() -> Result<(), String> {
        let maintenance = Maintenance::default();
        let notify = Arc::new(tokio::sync::Notify::new());
        let signal = Arc::clone(&notify);
        maintenance.start_action(
            Arc::new(move || {
                signal.notify_one();
                Ok(lkjmc_store::sync::RetentionResult {
                    archived: 2,
                    deleted: 1,
                })
            }),
            Duration::from_secs(60),
        )?;
        assert!(maintenance
            .start_action(Arc::new(|| unreachable!()), INTERVAL)
            .is_err());
        tokio::time::timeout(Duration::from_secs(1), notify.notified())
            .await
            .map_err(|_| "maintenance did not run".to_string())?;
        maintenance.shutdown().await?;
        let diagnostics = maintenance.diagnostics();
        assert_eq!(diagnostics.singleton_count, 1);
        assert_eq!(diagnostics.completed_runs, 1);
        assert_eq!(diagnostics.last_successful_run, Some(1));
        assert_eq!(diagnostics.archived_rows, 2);
        assert_eq!(diagnostics.deleted_rows, 1);
        assert!(!diagnostics.running);
        Ok(())
    }
}
