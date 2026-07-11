use std::collections::BTreeSet;

use super::adapters::ProcessAdapter;
use super::control::{
    Boundary, DeterministicClock, Failpoints, SeededSchedule, MAX_TRANSCRIPT_ITEMS,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ScenarioState {
    pub(super) order: Vec<String>,
    pub(super) started: Vec<String>,
    pub(super) observed: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FailureTranscript {
    pub(super) seed: u64,
    pub(super) clock_ms: u64,
    pub(super) boundary: Boundary,
    pub(super) hits: Vec<Boundary>,
    pub(super) state: ScenarioState,
}

pub(super) struct ScenarioRunner {
    seed: u64,
    clock: DeterministicClock,
    schedule: SeededSchedule,
    faults: Failpoints,
    durable_requests: BTreeSet<String>,
    plugin_connected: bool,
}

impl ScenarioRunner {
    pub(super) fn new(seed: u64, now_ms: u64) -> Self {
        Self {
            seed,
            clock: DeterministicClock::at(now_ms),
            schedule: SeededSchedule::new(seed),
            faults: Failpoints::default(),
            durable_requests: BTreeSet::new(),
            plugin_connected: true,
        }
    }

    pub(super) fn arm(&mut self, boundary: Boundary) {
        self.faults.arm(boundary);
    }

    pub(super) fn duplicate_request(&mut self, request_id: &str) -> bool {
        !self.durable_requests.insert(request_id.to_string())
    }

    pub(super) fn restart(&self) -> Self {
        Self {
            seed: self.seed,
            clock: self.clock,
            schedule: self.schedule,
            faults: Failpoints::default(),
            durable_requests: self.durable_requests.clone(),
            plugin_connected: self.plugin_connected,
        }
    }

    pub(super) fn has_durable_request(&self, request_id: &str) -> bool {
        self.durable_requests.contains(request_id)
    }

    pub(super) fn database_delay_exceeds(&mut self, limit_ms: u64, delay_ms: u64) -> bool {
        self.clock.advance(delay_ms);
        self.clock.now_ms() > limit_ms
    }

    pub(super) fn plugin_disconnect(&mut self) {
        self.plugin_connected = false;
    }

    pub(super) fn acknowledge_plugin(&self) -> bool {
        self.plugin_connected
    }

    pub(super) fn run_instances(&self, hung: &str, instances: &[&str]) -> Vec<String> {
        self.schedule
            .order(instances)
            .into_iter()
            .filter(|id| id != hung)
            .take(MAX_TRANSCRIPT_ITEMS)
            .collect()
    }

    pub(super) fn run_seeded_instances(
        &mut self,
        instances: &[&str],
    ) -> Result<ScenarioState, FailureTranscript> {
        let order = self
            .schedule
            .order(instances)
            .into_iter()
            .take(MAX_TRANSCRIPT_ITEMS)
            .collect::<Vec<_>>();
        let (failure, state) = {
            let mut process = ProcessAdapter::new(&mut self.faults);
            let mut failure = None;
            for instance in &order {
                if let Err(boundary) = process.start(instance) {
                    failure = Some(boundary);
                    break;
                }
            }
            let state = ScenarioState {
                order,
                started: process.processes(),
                observed: process.observations(),
            };
            (failure, state)
        };
        match failure {
            Some(boundary) => Err(FailureTranscript {
                seed: self.seed,
                clock_ms: self.clock.now_ms(),
                boundary,
                hits: self.faults.hits().to_vec(),
                state,
            }),
            None => Ok(state),
        }
    }
}
