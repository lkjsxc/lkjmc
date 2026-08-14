mod digest;
use crate::config::{NetworkConfig, NetworkRuntime};
use crate::instance::DesiredState;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkObservation {
    pub intent_digest: Option<String>,
    pub forwarding_secret_ready: bool,
    pub resources: BTreeMap<String, ResourceObservation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceObservation {
    pub spec_digest: String,
    pub runtime_present: bool,
    pub ready: bool,
    pub blocked: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkInspection {
    pub outcome: InspectionOutcome,
    pub intent_digest: String,
    pub changes: Vec<NetworkChange>,
    pub unsupported: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InspectionOutcome {
    Blocked,
    Changes,
    NoOp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkChange {
    pub order: u32,
    pub instance_id: Option<String>,
    pub action: ChangeAction,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChangeAction {
    VerifyAsset,
    EnsureSecret,
    Render,
    Start,
    Stop,
    VerifyReadiness,
}

pub fn inspect(intent: &NetworkConfig, observed: &NetworkObservation) -> NetworkInspection {
    let intent_digest = intent.digest();
    let mut unsupported = unsupported(intent);
    unsupported.extend(
        observed
            .resources
            .values()
            .filter_map(|resource| resource.blocked.clone()),
    );
    unsupported.sort();
    unsupported.dedup();
    if !unsupported.is_empty() {
        return NetworkInspection {
            outcome: InspectionOutcome::Blocked,
            intent_digest,
            changes: vec![],
            unsupported,
        };
    }
    let mut pending = Vec::new();
    if observed.intent_digest.as_deref() != Some(intent_digest.as_str()) {
        for asset in intent.assets.iter().filter(|asset| asset.required) {
            pending.push((
                asset.id.clone(),
                None,
                ChangeAction::VerifyAsset,
                "required asset digest must match",
            ));
        }
    }
    if !observed.forwarding_secret_ready {
        pending.push((
            String::new(),
            None,
            ChangeAction::EnsureSecret,
            "forwarding secret must exist privately",
        ));
    }
    let mut instances = intent.instances.iter().collect::<Vec<_>>();
    instances.sort_by(|left, right| left.id.cmp(&right.id));
    for instance in instances {
        let digest = intent.resource_digest(&instance.id);
        let observed_resource = observed.resources.get(&instance.id);
        let drift = observed_resource.is_none_or(|item| item.spec_digest != digest);
        if drift {
            pending.push((
                String::new(),
                Some(instance.id.clone()),
                ChangeAction::Render,
                "rendered configuration differs",
            ));
            if instance.desired_state == DesiredState::Running
                && observed_resource.is_some_and(|item| item.runtime_present)
            {
                pending.push((
                    String::new(),
                    Some(instance.id.clone()),
                    ChangeAction::Stop,
                    "drifted runtime must stop before replacement",
                ));
            }
        }
        match instance.desired_state {
            DesiredState::Running if drift || observed_resource.is_none_or(|item| !item.ready) => {
                pending.push((
                    String::new(),
                    Some(instance.id.clone()),
                    ChangeAction::Start,
                    "instance is not observed ready",
                ));
                pending.push((
                    String::new(),
                    Some(instance.id.clone()),
                    ChangeAction::VerifyReadiness,
                    "listener and runtime identity require observation",
                ));
            }
            DesiredState::Stopped if observed_resource.is_some_and(|item| item.runtime_present) => {
                pending.push((
                    String::new(),
                    Some(instance.id.clone()),
                    ChangeAction::Stop,
                    "instance is observed running",
                ));
            }
            _ => {}
        }
    }
    pending.sort_by(|a, b| (rank(a.2), &a.1, &a.0).cmp(&(rank(b.2), &b.1, &b.0)));
    let changes = pending
        .into_iter()
        .enumerate()
        .map(|(index, (_, id, action, reason))| NetworkChange {
            order: index as u32,
            instance_id: id,
            action,
            reason: reason.to_string(),
        })
        .collect::<Vec<_>>();
    let outcome = if changes.is_empty() {
        InspectionOutcome::NoOp
    } else {
        InspectionOutcome::Changes
    };
    NetworkInspection {
        outcome,
        intent_digest,
        changes,
        unsupported,
    }
}

fn unsupported(intent: &NetworkConfig) -> Vec<String> {
    let mut reasons = Vec::new();
    if intent.capabilities.runtime == NetworkRuntime::Kubernetes {
        reasons.push("kubernetes capability unsupported: runtime effects".to_string());
    }
    if !intent.capabilities.mounted_config {
        reasons.push("network capability unsupported: mounted-config".to_string());
    }
    if !intent.capabilities.mounted_secrets {
        reasons.push("network capability unsupported: mounted-secrets".to_string());
    }
    if !intent.capabilities.mounted_assets {
        reasons.push("network capability unsupported: mounted-assets".to_string());
    }
    for instance in intent
        .instances
        .iter()
        .filter(|item| item.desired_state == DesiredState::Running)
    {
        if instance.asset_ids.is_empty() {
            reasons.push(format!(
                "network capability unsupported: {} has no acquired immutable assets",
                instance.id
            ));
        }
    }
    reasons
}

fn rank(action: ChangeAction) -> u8 {
    match action {
        ChangeAction::EnsureSecret => 0,
        ChangeAction::VerifyAsset => 1,
        ChangeAction::Stop => 2,
        ChangeAction::Render => 3,
        ChangeAction::Start => 4,
        ChangeAction::VerifyReadiness => 5,
    }
}
