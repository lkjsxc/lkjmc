use std::collections::BTreeSet;

use crate::error::ConfigError;
use crate::instance::InstanceKind;

use super::network_intent::{
    InstanceIntegration, ListenerProtocol, NetworkConfig, ReadinessContract,
};
use super::validate::{
    require_kebab, require_non_empty, require_path, require_port, require_range,
};

impl NetworkConfig {
    pub(super) fn validate(&self) -> Result<(), ConfigError> {
        bounded("network.instances", self.instances.len(), 1, 64)?;
        bounded("network.routes", self.routes.len(), 1, 128)?;
        bounded("network.listeners", self.listeners.len(), 1, 64)?;
        bounded("network.assets", self.assets.len(), 0, 256)?;
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
            bounded(
                "network.listeners.publicHosts",
                listener.public_hosts.len(),
                0,
                32,
            )?;
            if !sockets.insert((
                listener.protocol,
                listener.bind_host.as_str(),
                listener.port,
            )) {
                return invalid("network.listeners", "duplicate protocol socket");
            }
            for host in &listener.public_hosts {
                require_non_empty("network.listeners.publicHosts", host)?;
            }
        }
        let mut velocity = 0;
        let mut used_assets = BTreeSet::new();
        for instance in &self.instances {
            require_range("network.instances.memoryMb", instance.memory_mb, 128, 65536)?;
            bounded(
                "network.instances.assetIds",
                instance.asset_ids.len(),
                0,
                32,
            )?;
            if !listeners.contains(instance.listener.as_str()) {
                return invalid(
                    "network.instances.listener",
                    "references an unknown listener",
                );
            }
            if instance.kind == InstanceKind::Velocity {
                velocity += 1;
            }
            validate_instance_contract(instance.kind, instance.integration, instance.readiness)?;
            for asset in &instance.asset_ids {
                if !assets.contains(asset.as_str()) {
                    return invalid("network.instances.assetIds", "references an unknown asset");
                }
                used_assets.insert(asset.as_str());
            }
        }
        if velocity != 1 {
            return invalid(
                "network.instances",
                "must contain exactly one velocity instance",
            );
        }
        for route in &self.routes {
            if !listeners.contains(route.listener.as_str())
                || !instances.contains(route.target.as_str())
            {
                return invalid("network.routes", "references an unknown listener or target");
            }
            bounded("network.routes.fallbacks", route.fallbacks.len(), 0, 16)?;
            for fallback in &route.fallbacks {
                if fallback == &route.target || !instances.contains(fallback.as_str()) {
                    return invalid(
                        "network.routes.fallbacks",
                        "must reference a distinct instance",
                    );
                }
            }
        }
        let mut digests = BTreeSet::new();
        for asset in &self.assets {
            if !used_assets.contains(asset.id.as_str()) {
                return invalid("network.assets", "contains an unreferenced asset");
            }
            require_path("network.assets.path", &asset.path)?;
            if asset.sha256.len() != 64
                || !asset.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                return invalid("network.assets.sha256", "must be 64 hexadecimal characters");
            }
            if fake_digest(&asset.sha256) {
                return invalid(
                    "network.assets.sha256",
                    "placeholder or repeated digests are forbidden",
                );
            }
            if !digests.insert(asset.sha256.to_ascii_lowercase()) {
                return invalid(
                    "network.assets.sha256",
                    "one digest cannot identify multiple assets",
                );
            }
        }
        require_path(
            "network.forwarding.secretFile",
            &self.forwarding.secret_file,
        )?;
        if !self
            .listeners
            .iter()
            .any(|x| x.protocol == ListenerProtocol::JavaTcp)
        {
            return invalid("network.listeners", "must contain a java-tcp listener");
        }
        Ok(())
    }
}

fn validate_instance_contract(
    kind: InstanceKind,
    integration: InstanceIntegration,
    readiness: ReadinessContract,
) -> Result<(), ConfigError> {
    let valid = match kind {
        InstanceKind::Velocity => {
            integration == InstanceIntegration::Velocity
                && readiness == ReadinessContract::VelocityStatus
        }
        InstanceKind::Paper | InstanceKind::Folia | InstanceKind::Purpur => {
            integration == InstanceIntegration::PaperCompatible
                && readiness == ReadinessContract::PluginHeartbeat
        }
        InstanceKind::VanillaCustom | InstanceKind::ModdedCustom => {
            integration == InstanceIntegration::None && readiness == ReadinessContract::Unsupported
        }
    };
    if valid {
        Ok(())
    } else {
        invalid(
            "network.instances",
            "kind, integration, and readiness contract disagree",
        )
    }
}

fn fake_digest(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [1, 2, 4, 8, 16, 32].into_iter().any(|width| {
        lower
            .as_bytes()
            .chunks(width)
            .all(|chunk| chunk == &lower.as_bytes()[..width])
    }) || lower == "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        || lower == "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
}

fn unique_ids<'a>(
    field: &'static str,
    values: impl Iterator<Item = &'a String>,
) -> Result<BTreeSet<&'a str>, ConfigError> {
    let mut seen = BTreeSet::new();
    for value in values {
        require_kebab(field, value)?;
        if !seen.insert(value.as_str()) {
            return invalid(field, "ids must be unique");
        }
    }
    Ok(seen)
}

fn bounded(field: &'static str, value: usize, min: usize, max: usize) -> Result<(), ConfigError> {
    if (min..=max).contains(&value) {
        Ok(())
    } else {
        invalid(field, format!("must contain {min}..={max} items"))
    }
}

fn invalid<T>(field: &'static str, message: impl Into<String>) -> Result<T, ConfigError> {
    Err(ConfigError::invalid(field, message.into()))
}
