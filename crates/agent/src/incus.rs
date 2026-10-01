use crate::config::{Binding, Config};
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use std::{path::Path, process::Stdio, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    process::Command,
};

#[derive(Clone)]
pub struct Incus {
    pub config: Config,
}
impl Incus {
    pub async fn verify_network(&self) -> Result<()> {
        let network = self
            .json(
                &self.config.tenant_project,
                &[
                    "query".into(),
                    format!(
                        "/1.0/networks/{}?project={}",
                        self.config.tenant_network, self.config.tenant_project
                    ),
                ],
            )
            .await?;
        ensure!(
            network["type"] == "bridge"
                && network["config"]["user.lkjmc.scope"] == "rebuild"
                && network["config"]["security.acls"] == self.config.tenant_acl
                && network["config"]["ipv6.address"] == "none"
                && network["config"]["security.acls.default.ingress.action"] == "reject"
                && network["config"]["security.acls.default.egress.action"] == "reject"
                && network["config"]["ipv4.firewall"] != "false",
            "Tenant network does not match the isolated rebuild network"
        );
        let acl = self
            .json(
                &self.config.tenant_project,
                &[
                    "query".into(),
                    format!(
                        "/1.0/network-acls/{}?project={}",
                        self.config.tenant_acl, self.config.tenant_project
                    ),
                ],
            )
            .await?;
        verify_acl(
            &acl,
            &self.config.proxy_addresses,
            &self.config.monitor_addresses,
        )
    }
    pub async fn run(
        &self,
        project: &str,
        args: &[String],
        input: Option<Vec<u8>>,
    ) -> Result<Vec<u8>> {
        crate::config::name(project)?;
        let mut child = Command::new(&self.config.incus)
            .args(["--force-local", "--project", project])
            .args(args)
            .stdin(if input.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .context("start Incus client")?;
        if let Some(input) = input {
            let mut stdin = child.stdin.take().unwrap();
            tokio::spawn(async move {
                let _ = stdin.write_all(&input).await;
            });
        }
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let (stdout, stderr, status) = tokio::time::timeout(
            Duration::from_secs(if args.first().is_some_and(|s| s == "export") {
                3600
            } else {
                180
            }),
            async {
                tokio::try_join!(
                    bounded(stdout, 4 * 1024 * 1024),
                    bounded(stderr, 128 * 1024),
                    async { Ok::<_, anyhow::Error>(child.wait().await?) }
                )
            },
        )
        .await
        .context("Incus operation timed out; reconcile before retry")??;
        ensure!(
            status.success(),
            "Incus operation failed: {}",
            String::from_utf8_lossy(&stderr)
                .chars()
                .take(2000)
                .collect::<String>()
        );
        Ok(stdout)
    }
    pub async fn json(&self, project: &str, args: &[String]) -> Result<Value> {
        Ok(serde_json::from_slice(
            &self.run(project, args, None).await?,
        )?)
    }
    pub async fn instance(&self, b: &Binding) -> Result<Value> {
        let all = self
            .json(&b.project, &["list".into(), "--format=json".into()])
            .await?;
        all.as_array()
            .context("Invalid Incus instance list")?
            .iter()
            .find(|v| v["name"] == b.instance)
            .cloned()
            .context("Managed instance is absent")
    }
    pub fn verify(&self, b: &Binding, value: &Value) -> Result<()> {
        ensure!(
            value["name"] == b.instance
                && value["expanded_config"]["user.lkjmc.server-id"] == b.server_id.to_string()
                && value["expanded_config"]["user.lkjmc.scope"] == "rebuild",
            "Instance ownership does not match this deployment"
        );
        if b.custom {
            ensure!(
                value["type"] == "virtual-machine" && b.project == self.config.tenant_project,
                "Untrusted servers must have their own VM"
            );
            let config = value["expanded_config"]
                .as_object()
                .context("Missing VM configuration")?;
            ensure!(
                !config.keys().any(|k| k.starts_with("raw.")),
                "Raw hypervisor configuration is outside tenant scope"
            );
            let devices = value["expanded_devices"]
                .as_object()
                .context("Missing device inventory")?;
            ensure!(
                devices.len() == 2,
                "Tenant VM must have exactly its own root disk and isolated NIC"
            );
            let disk = devices.get("root").context("Missing root disk")?;
            let nic = devices.get("eth0").context("Missing isolated NIC")?;
            ensure!(
                disk["type"] == "disk"
                    && disk["path"] == "/"
                    && disk["pool"] == self.config.storage_pool
                    && disk.get("source").is_none_or(|v| v == ""),
                "Unexpected tenant disk or host mount"
            );
            ensure!(
                nic["type"] == "nic"
                    && nic["network"] == self.config.tenant_network
                    && nic["ipv4.address"] == b.address.to_string()
                    && nic["security.mac_filtering"] == "true"
                    && nic["security.ipv4_filtering"] == "true"
                    && nic["security.ipv6_filtering"] == "true"
                    && nic["security.port_isolation"] == "true",
                "Tenant NIC isolation does not match the approved profile"
            );
            ensure!(
                nic.get("security.acls").is_none_or(|v| v == "")
                    && nic
                        .get("security.acls.default.ingress.action")
                        .is_none_or(|v| v == "reject")
                    && nic
                        .get("security.acls.default.egress.action")
                        .is_none_or(|v| v == "reject"),
                "Tenant NIC must inherit the network ACL without overrides"
            );
        }
        Ok(())
    }
    pub async fn exec(
        &self,
        b: &Binding,
        args: &[String],
        input: Option<Vec<u8>>,
    ) -> Result<Vec<u8>> {
        self.verify(b, &self.instance(b).await?)?;
        let mut command = vec![
            "exec".into(),
            b.instance.clone(),
            "--mode=non-interactive".into(),
            "--".into(),
        ];
        command.extend_from_slice(args);
        self.run(&b.project, &command, input).await
    }
    pub async fn helper(&self, b: &Binding, command: &str, value: Value) -> Result<Value> {
        let result: Value = serde_json::from_slice(
            &self
                .exec(
                    b,
                    &["/usr/local/lib/lkjmc/guest.py".into(), command.into()],
                    Some(serde_json::to_vec(&value)?),
                )
                .await?,
        )?;
        if let Some(error) = result["error"].as_str() {
            return Err(GuestFailure {
                message: error.chars().take(2000).collect(),
                no_effect: result["effect"] == "none",
            }
            .into());
        }
        Ok(result)
    }
    pub async fn push(&self, b: &Binding, source: &Path, destination: &str) -> Result<()> {
        self.verify(b, &self.instance(b).await?)?;
        ensure!(
            destination.starts_with("/var/lib/lkjmc/incoming/")
                || destination == "/usr/local/lib/lkjmc/guest.py",
            "Unmanaged push destination"
        );
        self.run(
            &b.project,
            &[
                "file".into(),
                "push".into(),
                "--create-dirs".into(),
                "--mode=0700".into(),
                source.to_string_lossy().into_owned(),
                format!("{}{destination}", b.instance),
            ],
            None,
        )
        .await?;
        Ok(())
    }
    pub async fn power(&self, b: &Binding, running: bool) -> Result<()> {
        let current = self.instance(b).await?;
        self.verify(b, &current)?;
        if current["status"] == if running { "Running" } else { "Stopped" } {
            return Ok(());
        }
        let mut args = vec![
            if running {
                "start".into()
            } else {
                "stop".into()
            },
            b.instance.clone(),
        ];
        if !running {
            args.extend(["--timeout=90".into()]);
        }
        self.run(&b.project, &args, None).await?;
        Ok(())
    }
}

async fn bounded(reader: impl AsyncRead + Unpin, limit: u64) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.take(limit + 1).read_to_end(&mut bytes).await?;
    ensure!(
        bytes.len() as u64 <= limit,
        "Guest or Incus output exceeded its limit"
    );
    Ok(bytes)
}
#[derive(Debug)]
pub struct GuestFailure {
    pub message: String,
    pub no_effect: bool,
}
impl std::fmt::Display for GuestFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl std::error::Error for GuestFailure {}

pub fn expected_acl(proxy: &[std::net::Ipv4Addr], monitor: &[std::net::Ipv4Addr]) -> Value {
    use serde_json::json;
    let mut ingress = Vec::new();
    for address in proxy.iter().chain(monitor) {
        ingress.push(json!({"action":"allow","state":"enabled","source":address.to_string(),"protocol":"tcp","destination_port":"25565"}));
    }
    let mut egress = Vec::new();
    for cidr in [
        "0.0.0.0/8",
        "10.0.0.0/8",
        "100.64.0.0/10",
        "127.0.0.0/8",
        "169.254.0.0/16",
        "172.16.0.0/12",
        "192.0.0.0/24",
        "192.168.0.0/16",
        "198.18.0.0/15",
        "224.0.0.0/4",
        "240.0.0.0/4",
        "::/0",
    ] {
        egress.push(json!({"action":"drop","state":"enabled","destination":cidr}));
    }
    for protocol in ["tcp", "udp", "icmp4"] {
        egress.push(json!({"action":"allow","state":"enabled","destination":"0.0.0.0/0","protocol":protocol}));
    }
    json!({"ingress":ingress,"egress":egress})
}
fn verify_acl(
    acl: &Value,
    proxy: &[std::net::Ipv4Addr],
    monitor: &[std::net::Ipv4Addr],
) -> Result<()> {
    let expected = expected_acl(proxy, monitor);
    for direction in ["ingress", "egress"] {
        fn normalize(value: &Value) -> Result<Vec<String>> {
            let mut result = Vec::new();
            for rule in value.as_array().context("Missing ACL rules")? {
                let mut normalized = serde_json::Map::new();
                for key in [
                    "action",
                    "state",
                    "source",
                    "destination",
                    "protocol",
                    "source_port",
                    "destination_port",
                    "icmp_type",
                    "icmp_code",
                ] {
                    let value =
                        rule[key]
                            .as_str()
                            .unwrap_or(if key == "state" { "enabled" } else { "" });
                    if !value.is_empty() {
                        normalized.insert(
                            key.into(),
                            Value::String(if key == "state" && value == "logged" {
                                "enabled".into()
                            } else {
                                value.into()
                            }),
                        );
                    }
                }
                result.push(serde_json::to_string(&normalized)?);
            }
            result.sort();
            result.dedup();
            Ok(result)
        }
        ensure!(
            normalize(&acl[direction])? == normalize(&expected[direction])?,
            "Tenant ACL {direction} does not enforce the expected private-network and ingress restrictions"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_destinations_and_ingress_cannot_be_weakened() {
        let proxy = ["10.203.61.10".parse().unwrap()];
        let monitor = ["10.203.62.1".parse().unwrap()];
        let mut acl = expected_acl(&proxy, &monitor);
        verify_acl(&acl, &proxy, &monitor).unwrap();
        acl["egress"]
            .as_array_mut()
            .unwrap()
            .retain(|r| r["destination"] != "10.0.0.0/8");
        assert!(verify_acl(&acl, &proxy, &monitor).is_err());
        let mut acl = expected_acl(&proxy, &monitor);
        acl["ingress"][0]["source"] = serde_json::json!("0.0.0.0/0");
        assert!(verify_acl(&acl, &proxy, &monitor).is_err());
        let mut acl = expected_acl(&proxy, &monitor);
        acl["egress"][0]["state"] = serde_json::json!("disabled");
        assert!(verify_acl(&acl, &proxy, &monitor).is_err());
    }
}
