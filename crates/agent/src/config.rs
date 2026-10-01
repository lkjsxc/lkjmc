use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    net::Ipv4Addr,
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub core_url: String,
    pub credential_file: PathBuf,
    pub state_dir: PathBuf,
    pub operations_lock: PathBuf,
    pub incus: PathBuf,
    pub tenant_project: String,
    pub tenant_profile: String,
    pub tenant_network: String,
    pub tenant_acl: String,
    pub proxy_addresses: Vec<Ipv4Addr>,
    pub monitor_addresses: Vec<Ipv4Addr>,
    pub storage_pool: String,
    pub image_fingerprint: String,
    pub forwarding_secret_file: PathBuf,
    pub addresses: Vec<Ipv4Addr>,
    pub max_tenant_memory_mib: u64,
    pub max_tenant_storage_mib: u64,
    pub host_memory_reserve_mib: u64,
    pub presets: Vec<Preset>,
    #[serde(default)]
    pub trusted_servers: BTreeMap<Uuid, Binding>,
    #[serde(default)]
    pub development: bool,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preset {
    pub software: String,
    pub version: String,
    pub url: String,
    pub sha256: String,
    pub java: u32,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub server_id: Uuid,
    pub project: String,
    pub instance: String,
    pub address: Ipv4Addr,
    pub custom: bool,
}
impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let value: Self =
            serde_json::from_slice(&std::fs::read(path).context("read agent configuration")?)?;
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<()> {
        let url = reqwest::Url::parse(&self.core_url)?;
        ensure!(
            url.username().is_empty()
                && url.password().is_none()
                && url.query().is_none()
                && url.path() == "/",
            "Core URL must be an origin"
        );
        ensure!(
            url.scheme() == "https"
                || self.development
                    && url.scheme() == "http"
                    && matches!(url.host_str(), Some("127.0.0.1" | "localhost")),
            "Non-local Core connections require HTTPS"
        );
        for p in [
            &self.state_dir,
            &self.operations_lock,
            &self.credential_file,
            &self.forwarding_secret_file,
            &self.incus,
        ] {
            ensure!(p.is_absolute(), "Agent paths must be absolute");
        }
        for s in [
            &self.tenant_project,
            &self.tenant_profile,
            &self.tenant_network,
            &self.tenant_acl,
            &self.storage_pool,
        ] {
            name(s)?;
        }
        ensure!(
            !self.proxy_addresses.is_empty() && !self.monitor_addresses.is_empty(),
            "Explicit ingress sources are required"
        );
        ensure!(
            self.tenant_project.starts_with("lkjmc-") && self.tenant_project != "lkjmc",
            "Agent needs the dedicated rebuild project"
        );
        ensure!(
            self.image_fingerprint.len() == 64
                && self
                    .image_fingerprint
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit()),
            "Pin the VM image fingerprint"
        );
        ensure!(
            !self.addresses.is_empty()
                && self.addresses.len()
                    == self
                        .addresses
                        .iter()
                        .collect::<std::collections::HashSet<_>>()
                        .len(),
            "VM address pool is empty or duplicated"
        );
        ensure!(
            self.max_tenant_memory_mib > 0 && self.max_tenant_storage_mib > 0,
            "Measured tenant capacity is required"
        );
        for (id, binding) in &self.trusted_servers {
            ensure!(
                *id == binding.server_id && !binding.custom,
                "Invalid trusted server binding"
            );
            name(&binding.project)?;
            name(&binding.instance)?;
            ensure!(
                binding.project.starts_with("lkjmc-") && binding.project != self.tenant_project,
                "Trusted instances require a separate rebuild project"
            );
        }
        for preset in &self.presets {
            let source = reqwest::Url::parse(&preset.url)?;
            ensure!(
                source.scheme() == "https"
                    && source.username().is_empty()
                    && source.password().is_none(),
                "Preset downloads require HTTPS"
            );
            ensure!(
                preset.sha256.len() == 64 && preset.sha256.bytes().all(|b| b.is_ascii_hexdigit()),
                "Preset requires SHA256"
            );
            ensure!(
                [8, 17, 21, 25].contains(&preset.java),
                "Unsupported Java runtime"
            );
        }
        Ok(())
    }
}
pub fn name(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty()
            && value.len() <= 63
            && value.as_bytes()[0].is_ascii_alphabetic()
            && value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-'),
        "Invalid managed resource name"
    );
    Ok(())
}
