use std::collections::BTreeMap;

use crate::config::{AssetKind, LkjmcConfig, NetworkAsset};
use crate::error::ConfigError;
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
    let first = inspect(&intent, &NetworkObservation::default());
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
    let resources = intent
        .instances
        .iter()
        .map(|instance| {
            (
                instance.id.clone(),
                ResourceObservation {
                    spec_digest: intent.resource_digest(&instance.id),
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
