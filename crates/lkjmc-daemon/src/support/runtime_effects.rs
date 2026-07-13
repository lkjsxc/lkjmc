use crate::app::AppState;
use crate::runtime::{RuntimeGoal, RuntimeObservation};

pub(crate) fn start_runtime(state: &AppState, id: &str) -> Result<RuntimeObservation, String> {
    crate::runtime::reconcile::reconcile(state, id, RuntimeGoal::Running, uuid::Uuid::new_v4())
}

#[cfg(test)]
#[path = "runtime_effects_tests.rs"]
mod tests;
