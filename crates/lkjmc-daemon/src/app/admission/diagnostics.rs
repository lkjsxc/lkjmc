use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::time::Instant;

use super::{Admission, WORKER_DEADLINE};

pub(crate) fn remaining_request_budget() -> Option<Duration> {
    WORKER_DEADLINE.with(|deadline| {
        deadline
            .get()
            .map(|value| value.saturating_duration_since(Instant::now()))
    })
}

impl Admission {
    pub(crate) fn diagnostics(&self) -> (bool, usize) {
        (
            !self.state.semaphore.is_closed(),
            self.state.in_flight.load(Ordering::Acquire),
        )
    }
}
