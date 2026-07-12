use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use tokio::sync::{Notify, OwnedSemaphorePermit, Semaphore};
use tokio::time::{timeout_at, Instant};

use crate::command_lifecycle::{ADMISSION_LIMIT, DEADLINE};

#[derive(Clone)]
pub(crate) struct Admission {
    state: Arc<State>,
}

pub(crate) struct RequestAdmission {
    lease: Arc<Lease>,
}

struct State {
    semaphore: Arc<Semaphore>,
    in_flight: AtomicUsize,
    idle: Notify,
}

struct Lease {
    _permit: OwnedSemaphorePermit,
    deadline: Instant,
    state: Arc<State>,
}

pub(crate) enum BlockingError {
    Deadline,
    Join,
}

impl Admission {
    pub(crate) fn new() -> Self {
        Self {
            state: Arc::new(State {
                semaphore: Arc::new(Semaphore::new(ADMISSION_LIMIT)),
                in_flight: AtomicUsize::new(0),
                idle: Notify::new(),
            }),
        }
    }

    pub(crate) fn try_admit(&self) -> Option<RequestAdmission> {
        let permit = self.state.semaphore.clone().try_acquire_owned().ok()?;
        self.state.in_flight.fetch_add(1, Ordering::AcqRel);
        Some(RequestAdmission {
            lease: Arc::new(Lease {
                _permit: permit,
                deadline: Instant::now() + DEADLINE,
                state: self.state.clone(),
            }),
        })
    }

    pub(crate) fn close(&self) {
        self.state.semaphore.close();
    }

    pub(crate) async fn wait_for_idle(&self) {
        loop {
            let notified = self.state.idle.notified();
            if self.state.in_flight.load(Ordering::Acquire) == 0 {
                return;
            }
            notified.await;
        }
    }
}

impl Clone for RequestAdmission {
    fn clone(&self) -> Self {
        Self {
            lease: self.lease.clone(),
        }
    }
}

impl RequestAdmission {
    pub(crate) fn deadline(&self) -> Instant {
        self.lease.deadline
    }

    pub(crate) async fn run_blocking<T, F>(&self, work: F) -> Result<T, BlockingError>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        if Instant::now() >= self.deadline() {
            return Err(BlockingError::Deadline);
        }
        let lease = self.lease.clone();
        let worker = tokio::task::spawn_blocking(move || {
            let _lease = lease;
            work()
        });
        match timeout_at(self.deadline(), worker).await {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(_)) => Err(BlockingError::Join),
            Err(_) => Err(BlockingError::Deadline),
        }
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        self.state.in_flight.fetch_sub(1, Ordering::AcqRel);
        self.state.idle.notify_waiters();
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn shutdown_waits_for_inflight_admission() -> Result<(), String> {
        let admission = Admission::new();
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
        admission.wait_for_idle().await;
        Ok(())
    }
}
