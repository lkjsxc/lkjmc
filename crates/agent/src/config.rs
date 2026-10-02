use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::OpenOptions,
    io::Read,
    net::Ipv4Addr,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub core_url: String,
    pub credential_file: PathBuf,
    pub state_dir: PathBuf,
    pub operations_lock: PathBuf,
    /// Root-owned deployment receipt tying runtime plans to the canonical host definition.
    pub management_manifest: Option<PathBuf>,
    pub incus: PathBuf,
    pub tenant_project: String,
    pub tenant_profile: String,
    pub tenant_network: String,
    pub tenant_acl: String,
    pub tenant_gateway: Ipv4Addr,
    pub proxy_addresses: Vec<Ipv4Addr>,
    pub monitor_addresses: Vec<Ipv4Addr>,
    pub storage_pool: String,
    pub storage_pool_path: PathBuf,
    pub image_fingerprint: String,
    pub forwarding_secret_file: PathBuf,
    pub addresses: Vec<Ipv4Addr>,
    pub max_tenant_memory_mib: u64,
    pub max_tenant_cpu: u64,
    pub max_tenant_storage_mib: u64,
    pub host_memory_reserve_mib: u64,
    pub pool_free_reserve_mib: u64,
    pub max_archive_mib: u64,
    pub presets: Vec<Preset>,
    #[serde(default)]
    pub trusted_servers: BTreeMap<Uuid, Binding>,
    #[serde(default)]
    pub development: bool,
}
#[derive(Clone, Serialize, Deserialize)]
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
        let mut file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)
            .context("read agent configuration")?;
        let metadata = file.metadata()?;
        ensure!(
            metadata.is_file() && metadata.nlink() == 1 && metadata.mode() & 0o022 == 0,
            "Agent configuration must be a protected regular file"
        );
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let value: Self = serde_json::from_slice(&bytes)?;
        if !value.development {
            ensure!(
                metadata.uid() == 0,
                "Production configuration must be owned by root"
            );
            crate::management::secure(path)?;
        }
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.development
                || self
                    .management_manifest
                    .as_ref()
                    .is_some_and(|p| p.is_absolute()),
            "Production requires a root management deployment receipt"
        );
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
            &self.storage_pool_path,
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
            self.tenant_gateway.octets()[0] == 10
                && self.tenant_gateway.octets()[3] == 1
                && self.addresses.iter().all(|ip| {
                    ip.octets()[..3] == self.tenant_gateway.octets()[..3]
                        && (2..=254).contains(&ip.octets()[3])
                }),
            "Tenant addresses must belong to the dedicated private /24"
        );
        ensure!(
            self.max_tenant_memory_mib > 0
                && self.max_tenant_cpu > 0
                && self.max_tenant_storage_mib > 0
                && self.max_archive_mib > 64
                && self.host_memory_reserve_mib > 0
                && self.pool_free_reserve_mib > 0,
            "Measured tenant capacity is required"
        );
        ensure!(
            self.development
                || self.storage_pool == "default"
                    && self.storage_pool_path == Path::new("/var/lib/incus/storage-pools/default")
                    && self.pool_free_reserve_mib >= 160 * 1024,
            "Production capacity must preserve the approved pool reserve"
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
