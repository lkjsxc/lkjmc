use std::collections::BTreeSet;

pub(super) const MAX_TRANSCRIPT_ITEMS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Boundary {
    BeforeTransactionCommit,
    AfterTransactionCommit,
    BeforeProcessEffect,
    AfterProcessEffect,
    BeforeObservation,
    HttpDeadline,
    CredentialLookup,
    BeforeShutdown,
}

impl Boundary {
    pub(super) const fn marker(self) -> &'static str {
        match self {
            Self::BeforeTransactionCommit => "fault-harness-before-transaction-commit",
            Self::AfterTransactionCommit => "fault-harness-after-transaction-commit",
            Self::BeforeProcessEffect => "fault-harness-before-process-effect",
            Self::AfterProcessEffect => "fault-harness-after-process-effect",
            Self::BeforeObservation => "fault-harness-before-observation",
            Self::HttpDeadline => "fault-harness-http-deadline",
            Self::CredentialLookup => "fault-harness-credential-lookup",
            Self::BeforeShutdown => "fault-harness-before-shutdown",
        }
    }
}

#[derive(Default)]
pub(super) struct Failpoints {
    armed: BTreeSet<Boundary>,
    hits: Vec<Boundary>,
}

impl Failpoints {
    pub(super) fn arm(&mut self, boundary: Boundary) {
        self.armed.insert(boundary);
    }

    pub(super) fn hit(&mut self, boundary: Boundary) -> Result<(), Boundary> {
        if self.hits.len() < MAX_TRANSCRIPT_ITEMS {
            self.hits.push(boundary);
        }
        if self.armed.remove(&boundary) {
            return Err(boundary);
        }
        Ok(())
    }

    pub(super) fn hits(&self) -> &[Boundary] {
        &self.hits
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DeterministicClock {
    now_ms: u64,
}

impl DeterministicClock {
    pub(super) const fn at(now_ms: u64) -> Self {
        Self { now_ms }
    }

    pub(super) fn advance(&mut self, elapsed_ms: u64) {
        self.now_ms = self.now_ms.saturating_add(elapsed_ms);
    }

    pub(super) const fn now_ms(self) -> u64 {
        self.now_ms
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SeededSchedule {
    seed: u64,
}

impl SeededSchedule {
    pub(super) const fn new(seed: u64) -> Self {
        Self { seed }
    }

    pub(super) fn order(self, instance_ids: &[&str]) -> Vec<String> {
        let mut ordered = instance_ids
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        ordered.sort();
        let mut state = self.seed;
        for index in (1..ordered.len()).rev() {
            let selected = (next_random(&mut state) % (index + 1) as u64) as usize;
            ordered.swap(index, selected);
        }
        ordered
    }
}

fn next_random(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut value = *state;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::{Boundary, SeededSchedule};

    #[test]
    fn every_boundary_has_a_release_inspection_marker() {
        let markers = [
            Boundary::BeforeTransactionCommit.marker(),
            Boundary::AfterTransactionCommit.marker(),
            Boundary::BeforeProcessEffect.marker(),
            Boundary::AfterProcessEffect.marker(),
            Boundary::BeforeObservation.marker(),
            Boundary::HttpDeadline.marker(),
            Boundary::CredentialLookup.marker(),
            Boundary::BeforeShutdown.marker(),
        ];
        assert_eq!(markers.len(), 8);
    }

    #[test]
    fn seeded_order_is_repeatable_and_seed_sensitive() {
        let instances = ["alpha", "beta", "gamma"];
        assert_eq!(
            SeededSchedule::new(41).order(&instances),
            SeededSchedule::new(41).order(&instances)
        );
        assert_ne!(
            SeededSchedule::new(41).order(&instances),
            SeededSchedule::new(42).order(&instances)
        );
    }
}
