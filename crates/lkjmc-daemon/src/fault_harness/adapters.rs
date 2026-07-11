use std::collections::BTreeSet;

use super::control::{Boundary, Failpoints, MAX_TRANSCRIPT_ITEMS};

pub(super) struct TransactionAdapter<'a> {
    faults: &'a mut Failpoints,
    committed: &'a mut BTreeSet<String>,
}

impl<'a> TransactionAdapter<'a> {
    pub(super) fn new(faults: &'a mut Failpoints, committed: &'a mut BTreeSet<String>) -> Self {
        Self { faults, committed }
    }

    pub(super) fn commit(&mut self, request_id: &str) -> Result<(), Boundary> {
        self.faults.hit(Boundary::BeforeTransactionCommit)?;
        self.committed.insert(request_id.to_string());
        self.faults.hit(Boundary::AfterTransactionCommit)
    }
}

pub(super) struct ProcessAdapter<'a> {
    faults: &'a mut Failpoints,
    processes: BTreeSet<String>,
    observations: BTreeSet<String>,
}

impl<'a> ProcessAdapter<'a> {
    pub(super) fn new(faults: &'a mut Failpoints) -> Self {
        Self {
            faults,
            processes: BTreeSet::new(),
            observations: BTreeSet::new(),
        }
    }

    pub(super) fn start(&mut self, instance_id: &str) -> Result<(), Boundary> {
        self.faults.hit(Boundary::BeforeProcessEffect)?;
        self.processes.insert(instance_id.to_string());
        self.faults.hit(Boundary::AfterProcessEffect)?;
        self.faults.hit(Boundary::BeforeObservation)?;
        self.observations.insert(instance_id.to_string());
        Ok(())
    }

    pub(super) fn has_process(&self, instance_id: &str) -> bool {
        self.processes.contains(instance_id)
    }

    pub(super) fn has_observation(&self, instance_id: &str) -> bool {
        self.observations.contains(instance_id)
    }

    pub(super) fn processes(&self) -> Vec<String> {
        self.processes
            .iter()
            .take(MAX_TRANSCRIPT_ITEMS)
            .cloned()
            .collect()
    }

    pub(super) fn observations(&self) -> Vec<String> {
        self.observations
            .iter()
            .take(MAX_TRANSCRIPT_ITEMS)
            .cloned()
            .collect()
    }
}

pub(super) struct ServiceAdapter<'a> {
    faults: &'a mut Failpoints,
}

impl<'a> ServiceAdapter<'a> {
    pub(super) fn new(faults: &'a mut Failpoints) -> Self {
        Self { faults }
    }

    pub(super) fn await_http(&mut self) -> Result<(), Boundary> {
        self.faults.hit(Boundary::HttpDeadline)
    }

    pub(super) fn credential(&mut self) -> Result<(), Boundary> {
        self.faults.hit(Boundary::CredentialLookup)
    }

    pub(super) fn shutdown(&mut self) -> Result<(), Boundary> {
        self.faults.hit(Boundary::BeforeShutdown)
    }
}
