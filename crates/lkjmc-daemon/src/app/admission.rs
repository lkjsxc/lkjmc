use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{oneshot, Notify, OwnedSemaphorePermit, Semaphore};
use tokio::time::{timeout_at, Instant};

use crate::command_lifecycle::{ADMISSION_LIMIT, DEADLINE};

thread_local! {
    static WORKER_DEADLINE: Cell<Option<Instant>> = const { Cell::new(None) };
}

#[derive(Clone)]
pub(crate) struct Admission {
    state: Arc<State>,
    deadline: Duration,
}

pub(crate) struct RequestAdmission {
    lease: Arc<Lease>,
}

struct State {
    semaphore: Arc<Semaphore>,
    in_flight: AtomicUsize,
    idle: Notify,
    workers: workers::Workers,
}

struct Lease {
    _permit: OwnedSemaphorePermit,
    deadline: Instant,
    state: Arc<State>,
}

struct DeadlineScope(Option<Instant>);

pub(crate) enum BlockingError {
    Deadline,
    Join,
}

pub(crate) fn remaining_request_budget() -> Option<Duration> {
    WORKER_DEADLINE.with(|deadline| {
        deadline
            .get()
            .map(|value| value.saturating_duration_since(Instant::now()))
    })
}

impl Admission {
    pub(crate) fn new() -> Self {
        Self::with_deadline(DEADLINE)
    }

    #[cfg(test)]
    pub(crate) fn with_deadline(deadline: Duration) -> Self {
        Self::build(deadline)
    }

    #[cfg(not(test))]
    fn with_deadline(deadline: Duration) -> Self {
        Self::build(deadline)
    }

    fn build(deadline: Duration) -> Self {
        Self {
            state: Arc::new(State {
                semaphore: Arc::new(Semaphore::new(ADMISSION_LIMIT)),
                in_flight: AtomicUsize::new(0),
                idle: Notify::new(),
                workers: workers::Workers::new(),
            }),
            deadline,
        }
    }

    pub(crate) fn try_admit(&self) -> Option<RequestAdmission> {
        let permit = self.state.semaphore.clone().try_acquire_owned().ok()?;
        self.state.in_flight.fetch_add(1, Ordering::AcqRel);
        Some(RequestAdmission {
            lease: Arc::new(Lease {
                _permit: permit,
                deadline: Instant::now() + self.deadline,
                state: self.state.clone(),
            }),
        })
    }

    pub(crate) fn close(&self) {
        self.state.semaphore.close();
    }

    pub(crate) async fn wait_for_idle(&self) -> Result<(), BlockingError> {
        let mut failed = false;
        loop {
            failed |= self.state.observe_finished().await.is_err();
            let notified = self.state.idle.notified();
            if self.state.in_flight.load(Ordering::Acquire) == 0 && self.state.no_workers() {
                return if failed {
                    Err(BlockingError::Join)
                } else {
                    Ok(())
                };
            }
            // A cancellable waiter never owns a live handle; retain it until
            // `is_finished` lets the next loop observe it without suspension.
            tokio::select! {
                _ = notified => {}
                _ = tokio::time::sleep(Duration::from_millis(1)) => {}
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn tracked_workers(&self) -> usize {
        self.state.tracked_workers()
    }

    #[cfg(test)]
    pub(crate) fn observed_workers(&self) -> usize {
        self.state.observed_workers()
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
        self.lease.state.observe_finished().await?;
        let (sender, receiver) = oneshot::channel();
        let (start, proceed) = std::sync::mpsc::sync_channel(1);
        let state = self.lease.state.clone();
        let worker_id = state.register_pending();
        let lease = self.lease.clone();
        let deadline = self.deadline();
        let worker = tokio::task::spawn_blocking(move || {
            let _lease = lease;
            if proceed.recv().is_err() {
                return;
            }
            let _deadline = DeadlineScope::enter(deadline);
            let output = work();
            let _ = sender.send(output);
        });
        state.attach(worker_id, worker);
        let _ = start.send(());
        let result = timeout_at(deadline, receiver).await;
        state.observe_finished().await?;
        match result {
            Ok(Ok(value)) if Instant::now() < deadline => Ok(value),
            Ok(Ok(_)) | Err(_) => Err(BlockingError::Deadline),
            Ok(Err(_)) => Err(BlockingError::Join),
        }
    }
}

impl DeadlineScope {
    fn enter(deadline: Instant) -> Self {
        Self(WORKER_DEADLINE.with(|current| current.replace(Some(deadline))))
    }
}

impl Drop for DeadlineScope {
    fn drop(&mut self) {
        WORKER_DEADLINE.with(|current| current.set(self.0));
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        self.state.in_flight.fetch_sub(1, Ordering::AcqRel);
        self.state.idle.notify_waiters();
    }
}

mod workers;

#[cfg(test)]
mod tests;
