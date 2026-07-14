use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::{NetworkConfig, NetworkRuntime};
use crate::instance::DesiredState;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkObservation {
    pub intent_digest: Option<String>,
    pub resources: BTreeMap<String, ResourceObservation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceObservation {
    pub spec_digest: String,
    pub ready: bool,
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
pub enum InspectionOutcome { Blocked, Changes, NoOp }

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
pub enum ChangeAction { VerifyAsset, EnsureSecret, Render, Start, Stop, VerifyReadiness }

pub fn inspect(intent: &NetworkConfig, observed: &NetworkObservation) -> NetworkInspection {
    let intent_digest = intent.digest();
    let unsupported = unsupported(intent);
    if !unsupported.is_empty() {
        return NetworkInspection { outcome: InspectionOutcome::Blocked, intent_digest, changes: vec![], unsupported };
    }
    let mut pending = Vec::new();
    if observed.intent_digest.as_deref() != Some(intent_digest.as_str()) {
        for asset in intent.assets.iter().filter(|asset| asset.required) {
            pending.push((asset.id.clone(), None, ChangeAction::VerifyAsset, "required asset digest must match"));
        }
        pending.push((String::new(), None, ChangeAction::EnsureSecret, "forwarding secret must exist privately"));
    }
    let mut instances = intent.instances.iter().collect::<Vec<_>>();
    instances.sort_by(|left, right| left.id.cmp(&right.id));
    for instance in instances {
        let digest = intent.resource_digest(&instance.id);
        let observed_resource = observed.resources.get(&instance.id);
        let drift = observed_resource.is_none_or(|item| item.spec_digest != digest);
        if drift {
            pending.push((String::new(), Some(instance.id.clone()), ChangeAction::Render, "rendered configuration differs"));
        }
        match instance.desired_state {
            DesiredState::Running if drift || observed_resource.is_none_or(|item| !item.ready) => {
                pending.push((String::new(), Some(instance.id.clone()), ChangeAction::Start, "instance is not observed ready"));
                pending.push((String::new(), Some(instance.id.clone()), ChangeAction::VerifyReadiness, "listener and runtime identity require observation"));
            }
            DesiredState::Stopped if observed_resource.is_some_and(|item| item.ready) => {
                pending.push((String::new(), Some(instance.id.clone()), ChangeAction::Stop, "instance is observed running"));
            }
            _ => {}
        }
    }
    pending.sort_by(|a, b| (rank(a.2), &a.1, &a.0).cmp(&(rank(b.2), &b.1, &b.0)));
    let changes = pending.into_iter().enumerate().map(|(index, (_, id, action, reason))| NetworkChange {
        order: index as u32, instance_id: id, action, reason: reason.to_string(),
    }).collect::<Vec<_>>();
    let outcome = if changes.is_empty() { InspectionOutcome::NoOp } else { InspectionOutcome::Changes };
    NetworkInspection { outcome, intent_digest, changes, unsupported }
}

fn unsupported(intent: &NetworkConfig) -> Vec<String> {
    if intent.capabilities.runtime == NetworkRuntime::Kubernetes {
        let mut missing = Vec::new();
        if !intent.capabilities.mounted_config { missing.push("mounted-config"); }
        if !intent.capabilities.mounted_secrets { missing.push("mounted-secrets"); }
        if !intent.capabilities.mounted_assets { missing.push("mounted-assets"); }
        return missing.into_iter().map(|item| format!("kubernetes capability unsupported: {item}")).collect();
    }
    Vec::new()
}

fn rank(action: ChangeAction) -> u8 {
    match action {
        ChangeAction::EnsureSecret => 0, ChangeAction::VerifyAsset => 1,
        ChangeAction::Render => 2, ChangeAction::Stop => 3,
        ChangeAction::Start => 4, ChangeAction::VerifyReadiness => 5,
    }
}

impl NetworkConfig {
    pub fn digest(&self) -> String { digest_json(&self.normalized()) }
    pub fn resource_digest(&self, id: &str) -> String {
        let instance = self.instances.iter().find(|item| item.id == id);
        let listener = instance.and_then(|item| self.listener(&item.listener));
        let routes = self.routes.iter().filter(|route| route.target == id || route.fallbacks.iter().any(|item| item == id)).collect::<Vec<_>>();
        digest_json(&(instance, listener, routes, &self.auth, &self.forwarding))
    }
    fn normalized(&self) -> Self {
        let mut value = self.clone();
        value.instances.sort_by(|a, b| a.id.cmp(&b.id));
        value.routes.sort_by(|a, b| a.id.cmp(&b.id));
        value.listeners.sort_by(|a, b| a.id.cmp(&b.id));
        value.assets.sort_by(|a, b| a.id.cmp(&b.id));
        for item in &mut value.instances { item.asset_ids.sort(); }
        value
    }
}

fn digest_json<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    format!("{:x}", Sha256::digest(bytes))
}
