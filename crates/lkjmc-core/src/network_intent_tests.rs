use std::collections::BTreeMap;

use crate::config::{AssetKind, LkjmcConfig, NetworkAsset};
use crate::error::ConfigError;
use crate::instance::DesiredState;
use crate::network_intent::{
    inspect, ChangeAction, InspectionOutcome, NetworkObservation, ResourceObservation,
};

fn config() -> Result<LkjmcConfig, ConfigError> {
    let mut config =
        LkjmcConfig::from_json_str(include_str!("../../../config/defaults/daemon.json.example"))?;
    config.network.assets = vec![
        asset(
            "folia-server",
            AssetKind::Server,
            "780ecd0728ca321d6421db8597a09b6d34c6e4f6dd622de86cad8412c6a12685",
        ),
        asset(
            "velocity-server",
            AssetKind::Server,
            "3f3d4f4cdaff94f0089cd7fe6f78acb7475c8ccdcfef4ae4f462b6549f3da747",
        ),
    ];
    config.network.instances[0].asset_ids = vec!["folia-server".to_string()];
    config.network.instances[1].asset_ids = vec!["velocity-server".to_string()];
    config.network.capabilities.mounted_assets = true;
    Ok(config)
}

fn asset(id: &str, kind: AssetKind, sha256: &str) -> NetworkAsset {
    NetworkAsset {
        id: id.to_string(),
        kind,
        path: format!("/tmp/{id}.jar"),
        sha256: sha256.to_string(),
        required: true,
    }
}

#[test]
fn inspect_is_exact_deterministic_and_reapply_is_noop() -> Result<(), ConfigError> {
    let intent = config()?.network;
    let absent_resources = intent
        .instances
        .iter()
        .map(|instance| {
            (
                instance.id.clone(),
                ResourceObservation {
                    spec_digest: "drift".to_string(),
                    runtime_present: false,
                    ready: false,
                    blocked: None,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let first = inspect(
        &intent,
        &NetworkObservation {
            intent_digest: None,
            forwarding_secret_ready: false,
            resources: absent_resources,
        },
    );
    assert_eq!(first.outcome, InspectionOutcome::Changes);
    assert!(first
        .changes
        .iter()
        .any(|item| item.action == ChangeAction::EnsureSecret));
    assert_eq!(
        first
            .changes
            .iter()
            .filter(|item| item.action == ChangeAction::Start)
            .count(),
        2
    );
    assert_eq!(
        first
            .changes
            .iter()
            .filter(|item| item.action == ChangeAction::Stop)
            .count(),
        0
    );
    let drifted_resources = intent
        .instances
        .iter()
        .map(|instance| {
            (
                instance.id.clone(),
                ResourceObservation {
                    spec_digest: "0".repeat(64),
                    runtime_present: true,
                    ready: true,
                    blocked: None,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let drifted = inspect(
        &intent,
        &NetworkObservation {
            intent_digest: None,
            forwarding_secret_ready: true,
            resources: drifted_resources,
        },
    );
    for instance in &intent.instances {
        let stop = drifted
            .changes
            .iter()
            .position(|change| {
                change.instance_id.as_deref() == Some(instance.id.as_str())
                    && change.action == ChangeAction::Stop
            })
            .ok_or_else(|| ConfigError::invalid("network.plan", "stop missing"))?;
        let render = drifted
            .changes
            .iter()
            .position(|change| {
                change.instance_id.as_deref() == Some(instance.id.as_str())
                    && change.action == ChangeAction::Render
            })
            .ok_or_else(|| ConfigError::invalid("network.plan", "render missing"))?;
        assert!(stop < render);
    }
    let resources = intent
        .instances
        .iter()
        .map(|instance| {
            (
                instance.id.clone(),
                ResourceObservation {
                    spec_digest: intent.resource_digest(&instance.id),
                    runtime_present: true,
                    ready: true,
                    blocked: None,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let observed = NetworkObservation {
        intent_digest: Some(intent.digest()),
        forwarding_secret_ready: true,
        resources,
    };
    assert_eq!(inspect(&intent, &observed).outcome, InspectionOutcome::NoOp);
    Ok(())
}

#[test]
fn stopped_intent_stops_owned_runtime_even_when_listener_is_not_ready() -> Result<(), ConfigError> {
    let mut intent = config()?.network;
    intent.instances[0].desired_state = DesiredState::Stopped;
    let stopped_id = intent.instances[0].id.clone();
    let resources = intent
        .instances
        .iter()
        .map(|instance| {
            (
                instance.id.clone(),
                ResourceObservation {
                    spec_digest: intent.resource_digest(&instance.id),
                    runtime_present: true,
                    ready: instance.id != stopped_id,
                    blocked: None,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let inspected = inspect(
        &intent,
        &NetworkObservation {
            intent_digest: Some(intent.digest()),
            forwarding_secret_ready: true,
            resources,
        },
    );
    assert_eq!(inspected.outcome, InspectionOutcome::Changes);
    assert_eq!(inspected.changes.len(), 1);
    assert_eq!(inspected.changes[0].action, ChangeAction::Stop);
    assert_eq!(
        inspected.changes[0].instance_id.as_deref(),
        Some(stopped_id.as_str())
    );
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
    assert_eq!(
        inspect(&intent, &Default::default()).changes,
        inspect(&reordered, &Default::default()).changes
    );
    Ok(())
}
