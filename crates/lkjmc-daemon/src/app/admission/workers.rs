use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use tokio::task::JoinHandle;

use super::{BlockingError, State};

pub(super) struct Workers {
    next: AtomicUsize,
    entries: Mutex<BTreeMap<usize, Worker>>,
    observed: AtomicUsize,
}

enum Worker {
    Pending,
    Running(JoinHandle<()>),
    Joining,
}

impl Workers {
    pub(super) fn new() -> Self {
        Self {
            next: AtomicUsize::new(0),
            entries: Mutex::new(BTreeMap::new()),
            observed: AtomicUsize::new(0),
        }
    }
}

impl State {
    pub(super) fn register_pending(&self) -> usize {
        let id = self.workers.next.fetch_add(1, Ordering::Relaxed);
        self.entries().insert(id, Worker::Pending);
        id
    }

    pub(super) fn attach(&self, id: usize, worker: JoinHandle<()>) {
        let mut entries = self.entries();
        let id = if matches!(entries.get(&id), Some(Worker::Pending)) {
            id
        } else {
            self.workers.next.fetch_add(1, Ordering::Relaxed)
        };
        entries.insert(id, Worker::Running(worker));
        drop(entries);
        self.idle.notify_waiters();
    }

    pub(super) async fn observe_finished(&self) -> Result<(), BlockingError> {
        let workers = self.take_finished();
        let mut failed = false;
        for (id, worker) in workers {
            failed |= worker.await.is_err();
            self.entries().remove(&id);
            self.workers.observed.fetch_add(1, Ordering::AcqRel);
        }
        self.idle.notify_waiters();
        if failed {
            Err(BlockingError::Join)
        } else {
            Ok(())
        }
    }

    pub(super) fn no_workers(&self) -> bool {
        self.entries().is_empty()
    }

    #[cfg(test)]
    pub(super) fn tracked_workers(&self) -> usize {
        self.entries().len()
    }

    #[cfg(test)]
    pub(super) fn observed_workers(&self) -> usize {
        self.workers.observed.load(Ordering::Acquire)
    }

    fn take_finished(&self) -> Vec<(usize, JoinHandle<()>)> {
        let mut entries = self.entries();
        let ids = entries
            .iter()
            .filter_map(|(id, worker)| match worker {
                Worker::Running(worker) if worker.is_finished() => Some(*id),
                _ => None,
            })
            .collect::<Vec<_>>();
        ids.into_iter()
            .filter_map(
                |id| match std::mem::replace(entries.get_mut(&id)?, Worker::Joining) {
                    Worker::Running(worker) => Some((id, worker)),
                    _ => None,
                },
            )
            .collect()
    }

    fn entries(&self) -> std::sync::MutexGuard<'_, BTreeMap<usize, Worker>> {
        self.workers
            .entries
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }
}
