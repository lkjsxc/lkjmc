use std::collections::BTreeSet;

use crate::error::ConfigError;
use crate::instance::InstanceKind;

use super::network_intent::{ListenerProtocol, NetworkConfig};
use super::validate::{require_kebab, require_non_empty, require_path, require_port, require_range};

impl NetworkConfig {
    pub(super) fn validate(&self) -> Result<(), ConfigError> {
        bounded("network.instances", self.instances.len(), 1, 64)?;
        bounded("network.routes", self.routes.len(), 1, 128)?;
        bounded("network.listeners", self.listeners.len(), 1, 64)?;
        bounded("network.assets", self.assets.len(), 1, 256)?;
        if self.revision == 0 {
            return invalid("network.revision", "must be positive");
        }
        let instances = unique_ids("network.instances", self.instances.iter().map(|x| &x.id))?;
        let listeners = unique_ids("network.listeners", self.listeners.iter().map(|x| &x.id))?;
        let assets = unique_ids("network.assets", self.assets.iter().map(|x| &x.id))?;
        unique_ids("network.routes", self.routes.iter().map(|x| &x.id))?;
        let mut sockets = BTreeSet::new();
        for listener in &self.listeners {
            require_non_empty("network.listeners.bindHost", &listener.bind_host)?;
            require_port("network.listeners.port", listener.port)?;
            bounded("network.listeners.publicHosts", listener.public_hosts.len(), 0, 32)?;
            if !sockets.insert((listener.protocol, listener.bind_host.as_str(), listener.port)) {
                return invalid("network.listeners", "duplicate protocol socket");
            }
            for host in &listener.public_hosts {
                require_non_empty("network.listeners.publicHosts", host)?;
            }
        }
        let mut velocity = 0;
        for instance in &self.instances {
            require_range("network.instances.memoryMb", instance.memory_mb, 128, 65536)?;
            bounded("network.instances.assetIds", instance.asset_ids.len(), 1, 32)?;
            if !listeners.contains(instance.listener.as_str()) {
                return invalid("network.instances.listener", "references an unknown listener");
            }
            if instance.kind == InstanceKind::Velocity { velocity += 1; }
            for asset in &instance.asset_ids {
                if !assets.contains(asset.as_str()) {
                    return invalid("network.instances.assetIds", "references an unknown asset");
                }
            }
        }
        if velocity != 1 { return invalid("network.instances", "must contain exactly one velocity instance"); }
        for route in &self.routes {
            if !listeners.contains(route.listener.as_str()) || !instances.contains(route.target.as_str()) {
                return invalid("network.routes", "references an unknown listener or target");
            }
            bounded("network.routes.fallbacks", route.fallbacks.len(), 0, 16)?;
            for fallback in &route.fallbacks {
                if fallback == &route.target || !instances.contains(fallback.as_str()) {
                    return invalid("network.routes.fallbacks", "must reference a distinct instance");
                }
            }
        }
        for asset in &self.assets {
            require_path("network.assets.path", &asset.path)?;
            if asset.sha256.len() != 64 || !asset.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return invalid("network.assets.sha256", "must be 64 hexadecimal characters");
            }
        }
        require_path("network.forwarding.secretFile", &self.forwarding.secret_file)?;
        if !self.listeners.iter().any(|x| x.protocol == ListenerProtocol::JavaTcp) {
            return invalid("network.listeners", "must contain a java-tcp listener");
        }
        Ok(())
    }
}

fn unique_ids<'a>(field: &'static str, values: impl Iterator<Item = &'a String>) -> Result<BTreeSet<&'a str>, ConfigError> {
    let mut seen = BTreeSet::new();
    for value in values {
        require_kebab(field, value)?;
        if !seen.insert(value.as_str()) { return invalid(field, "ids must be unique"); }
    }
    Ok(seen)
}

fn bounded(field: &'static str, value: usize, min: usize, max: usize) -> Result<(), ConfigError> {
    if (min..=max).contains(&value) { Ok(()) } else { invalid(field, format!("must contain {min}..={max} items")) }
}

fn invalid<T>(field: &'static str, message: impl Into<String>) -> Result<T, ConfigError> {
    Err(ConfigError::invalid(field, message.into()))
}
