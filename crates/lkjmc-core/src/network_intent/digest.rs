use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::config::NetworkConfig;

impl NetworkConfig {
    pub fn digest(&self) -> String {
        digest_json(&self.normalized())
    }

    pub fn resource_digest(&self, id: &str) -> String {
        let instance = self.instances.iter().find(|item| item.id == id);
        let listener = instance.and_then(|item| self.listener(&item.listener));
        let routes = self
            .routes
            .iter()
            .filter(|route| route.target == id || route.fallbacks.iter().any(|item| item == id))
            .collect::<Vec<_>>();
        digest_json(&(instance, listener, routes, &self.auth, &self.forwarding))
    }

    fn normalized(&self) -> Self {
        let mut value = self.clone();
        value.instances.sort_by(|a, b| a.id.cmp(&b.id));
        value.routes.sort_by(|a, b| a.id.cmp(&b.id));
        value.listeners.sort_by(|a, b| a.id.cmp(&b.id));
        value.assets.sort_by(|a, b| a.id.cmp(&b.id));
        for item in &mut value.instances {
            item.asset_ids.sort();
        }
        value
    }
}

fn digest_json<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    format!("{:x}", Sha256::digest(bytes))
}
