use std::collections::BTreeMap;

use crate::config::LkjmcConfig;
use crate::error::ConfigError;
use crate::network_intent::{inspect, ChangeAction, InspectionOutcome, NetworkObservation, ResourceObservation};

fn config() -> Result<LkjmcConfig, ConfigError> {
    LkjmcConfig::from_json_str(include_str!("../../../config/defaults/daemon.json.example"))
}

#[test]
fn inspect_is_exact_deterministic_and_reapply_is_noop() -> Result<(), ConfigError> {
    let intent = config()?.network;
    let first = inspect(&intent, &NetworkObservation::default());
    assert_eq!(first.outcome, InspectionOutcome::Changes);
    assert!(first.changes.iter().any(|item| item.action == ChangeAction::EnsureSecret));
    assert_eq!(first.changes.iter().filter(|item| item.action == ChangeAction::Start).count(), 2);
    let resources = intent.instances.iter().map(|instance| (
        instance.id.clone(),
        ResourceObservation { spec_digest: intent.resource_digest(&instance.id), ready: true },
    )).collect::<BTreeMap<_, _>>();
    let observed = NetworkObservation { intent_digest: Some(intent.digest()), resources };
    assert_eq!(inspect(&intent, &observed).outcome, InspectionOutcome::NoOp);
    Ok(())
}

#[test]
fn array_order_does_not_change_plan_identity() -> Result<(), ConfigError> {
    let intent = config()?.network;
    let mut reordered = intent.clone();
    reordered.instances.reverse();
    reordered.listeners.reverse();
    reordered.assets.reverse();
    assert_eq!(intent.digest(), reordered.digest());
    assert_eq!(inspect(&intent, &Default::default()).changes, inspect(&reordered, &Default::default()).changes);
    Ok(())
}
