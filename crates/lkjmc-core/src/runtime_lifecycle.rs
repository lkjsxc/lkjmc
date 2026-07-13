#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeIntent {
    Running,
    Stopped,
    Deleted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeObserved {
    Running,
    Absent,
    Unhealthy,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LifecycleInput {
    pub intent: RuntimeIntent,
    pub observed: RuntimeObserved,
    pub pending_operation: bool,
    pub capability_supported: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleDecision {
    Start,
    Stop,
    Delete,
    ObservePending,
    Noop,
    Unsupported,
}

pub fn decide(input: LifecycleInput) -> LifecycleDecision {
    if !input.capability_supported {
        return LifecycleDecision::Unsupported;
    }
    if input.pending_operation {
        return LifecycleDecision::ObservePending;
    }
    match (input.intent, input.observed) {
        (RuntimeIntent::Running, RuntimeObserved::Running) => LifecycleDecision::Noop,
        (RuntimeIntent::Running, _) => LifecycleDecision::Start,
        (RuntimeIntent::Stopped, RuntimeObserved::Absent) => LifecycleDecision::Noop,
        (RuntimeIntent::Stopped, _) => LifecycleDecision::Stop,
        (RuntimeIntent::Deleted, _) => LifecycleDecision::Delete,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_is_observed_before_repeating_effect() {
        let decision = decide(LifecycleInput {
            intent: RuntimeIntent::Running,
            observed: RuntimeObserved::Unknown,
            pending_operation: true,
            capability_supported: true,
        });
        assert_eq!(decision, LifecycleDecision::ObservePending);
    }

    #[test]
    fn satisfied_intent_is_idempotent() {
        let decision = decide(LifecycleInput {
            intent: RuntimeIntent::Running,
            observed: RuntimeObserved::Running,
            pending_operation: false,
            capability_supported: true,
        });
        assert_eq!(decision, LifecycleDecision::Noop);
    }
}
