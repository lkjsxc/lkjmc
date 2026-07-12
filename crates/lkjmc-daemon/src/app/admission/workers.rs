use tokio::task::JoinHandle;

use super::State;

impl State {
    pub(super) fn track(&self, worker: JoinHandle<()>) {
        self.reap_finished();
        let id = self
            .next_worker
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.workers
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .insert(id, worker);
    }

    pub(super) fn reap_finished(&self) {
        self.workers
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .retain(|_, worker| !worker.is_finished());
    }

    pub(super) fn take_workers(&self) -> Vec<JoinHandle<()>> {
        std::mem::take(
            &mut *self
                .workers
                .lock()
                .unwrap_or_else(|error| error.into_inner()),
        )
        .into_values()
        .collect()
    }

    pub(super) fn no_workers(&self) -> bool {
        self.workers
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .is_empty()
    }
}
