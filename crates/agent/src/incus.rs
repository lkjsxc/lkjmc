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
                && network["config"]["ipv4.address"]
                    == format!("{}/24", self.config.tenant_gateway)
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
            self.config.tenant_gateway,
        )
    }
    pub async fn run(
        &self,
        project: &str,
        args: &[String],
        input: Option<Vec<u8>>,
    ) -> Result<Vec<u8>> {
        self.run_inner(project, args, input, None).await
    }
    pub async fn export(&self, b: &Binding, destination: &Path, limit: u64) -> Result<()> {
        self.verify(b, &self.instance(b).await?)?;
        self.run_inner(
            &b.project,
            &[
                "export".into(),
                b.instance.clone(),
                destination.to_string_lossy().into_owned(),
                "--instance-only".into(),
            ],
            None,
            Some(limit),
        )
        .await?;
        Ok(())
    }
    async fn run_inner(
        &self,
        project: &str,
        args: &[String],
        input: Option<Vec<u8>>,
        file_limit: Option<u64>,
    ) -> Result<Vec<u8>> {
        crate::config::name(project)?;
        let mut command = Command::new(&self.config.incus);
        let scoped = scoped_args(project, args)?;
        if let Some(limit) = file_limit {
            bound_file_size(&mut command, limit);
        }
        let mut child = command
            .args(scoped)
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
        if matches!(
            command,
            "logs"
                | "files"
                | "file_read"
                | "file_write"
                | "file_delete"
                | "directory_create"
                | "operator"
                | "install"
                | "inspection_ready"
                | "inspection_open"
                | "inspection_close"
                | "inspection_release"
        ) {
            use sha2::{Digest, Sha256};
            let installed = self
                .exec(
                    b,
                    &[
                        "/usr/bin/sha256sum".into(),
                        "/usr/local/lib/lkjmc/guest.py".into(),
                    ],
                    None,
                )
                .await?;
            let policy = self
                .exec(
                    b,
                    &[
                        "/usr/bin/sha256sum".into(),
                        "/usr/local/lib/lkjmc/managed-paths.json".into(),
                    ],
                    None,
                )
                .await?;
            ensure!(
                String::from_utf8_lossy(&policy).split_whitespace().next()
                    == Some(
                        hex::encode(Sha256::digest(include_bytes!(
                            "../../../ops/guest/managed-paths.json"
                        )))
                        .as_str()
                    ),
                "Guest managed-path policy needs a reviewed upgrade"
            );
            let expected = hex::encode(Sha256::digest(include_bytes!(
                "../../../ops/guest/guest.py"
            )));
            if String::from_utf8_lossy(&installed)
                .split_whitespace()
                .next()
                != Some(expected.as_str())
            {
                return Err(GuestFailure {
                    message: "The guest filesystem helper needs a reviewed upgrade before this operation. Existing receipts must be reconciled during that upgrade.".into(),
                    no_effect: matches!(command, "logs" | "files" | "file_read"),
                }.into());
            }
        }
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
                || matches!(
                    destination,
                    "/usr/local/lib/lkjmc/guest.py" | "/usr/local/lib/lkjmc/managed-paths.json"
                ),
            "Unmanaged push destination"
        );
        self.run(
            &b.project,
            &[
                "file".into(),
                "push".into(),
                "--create-dirs".into(),
                format!(
                    "--mode={}",
                    if destination.ends_with("/guest.py") {
                        "0755"
                    } else if destination.ends_with("/managed-paths.json") {
                        "0644"
                    } else {
                        "0700"
                    }
                ),
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

fn scoped_args(project: &str, args: &[String]) -> Result<Vec<String>> {
    crate::config::name(project)?;
    let mut result = vec!["--force-local".into()];
    if args.first().is_some_and(|s| s == "query") {
        // Incus query rejects --project; the raw API URL owns its scope.
        ensure!(args.len() == 2, "Expected one project-scoped query URL");
        let url = reqwest::Url::parse(&format!("http://localhost{}", args[1]))?;
        ensure!(
            args[1].starts_with("/1.0/")
                && url
                    .query_pairs()
                    .filter(|(key, _)| key == "project")
                    .count()
                    == 1
                && url
                    .query_pairs()
                    .any(|(key, value)| key == "project" && value == project),
            "Incus query must explicitly select the requested project"
        );
    } else {
        result.extend(["--project".into(), project.into()]);
    }
    result.extend_from_slice(args);
    Ok(result)
}

fn bound_file_size(command: &mut Command, limit: u64) {
    // The daemon retains its own limits; pool preflight accounts for staging.
    // This bounds the client output even if the daemon streams excess bytes.
    unsafe {
        command.pre_exec(move || {
            let mut current = std::mem::MaybeUninit::<libc::rlimit>::uninit();
            if libc::getrlimit(libc::RLIMIT_FSIZE, current.as_mut_ptr()) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            let mut current = current.assume_init();
            current.rlim_cur = current.rlim_cur.min(limit as libc::rlim_t);
            if libc::setrlimit(libc::RLIMIT_FSIZE, &current) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

#[cfg(test)]
mod export_limit_tests {
    use super::*;
    #[tokio::test]
    async fn kernel_stops_an_export_writer_at_the_remaining_archive_budget() {
        let path = std::env::temp_dir().join(format!("lkjmc-file-limit-{}", uuid::Uuid::new_v4()));
        let mut command = Command::new("python3");
        command
            .args(["-c", "import sys; open(sys.argv[1],'wb').write(b'x'*65536)"])
            .arg(&path);
        bound_file_size(&mut command, 8192);
        assert!(!command.output().await.unwrap().status.success());
        assert_eq!(std::fs::metadata(&path).unwrap().len(), 8192);
        std::fs::remove_file(path).unwrap();
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

pub fn expected_acl(
    proxy: &[std::net::Ipv4Addr],
    monitor: &[std::net::Ipv4Addr],
    gateway: std::net::Ipv4Addr,
) -> Value {
    use serde_json::json;
    let mut ingress = Vec::new();
    for address in proxy.iter().chain(monitor) {
        ingress.push(json!({"action":"allow","state":"enabled","source":address.to_string(),"protocol":"tcp","destination_port":"25565"}));
    }
    for protocol in ["tcp", "udp"] {
        ingress.push(json!({"action":"allow","state":"enabled","source":gateway.to_string(),"protocol":protocol,"source_port":"53"}));
    }
    ingress.push(json!({"action":"allow","state":"enabled","source":gateway.to_string(),"protocol":"udp","source_port":"67","destination_port":"68"}));
    let mut egress = Vec::new();
    for cidr in [
        "0.0.0.0/8",
        "100.64.0.0/10",
        "127.0.0.0/8",
        "169.254.0.0/16",
        "172.16.0.0/12",
        "192.0.0.0/24",
        "192.168.0.0/16",
        "198.18.0.0/15",
        "224.0.0.0/4",
        "::/0",
    ] {
        egress.push(json!({"action":"drop","state":"enabled","destination":cidr}));
    }
    // Incus evaluates all drops before allows. Exclude only the bridge DNS/DHCP
    // address and DHCP broadcast from the private ranges, then close every
    // other protocol/port on those two exceptions.
    for cidr in cidrs_without("10.0.0.0".parse().unwrap(), 8, gateway)
        .into_iter()
        .chain(cidrs_without(
            "240.0.0.0".parse().unwrap(),
            4,
            std::net::Ipv4Addr::BROADCAST,
        ))
    {
        egress.push(json!({"action":"drop","state":"enabled","destination":cidr}));
    }
    for (destination, tcp, udp) in [
        (gateway.to_string(), "1-52,54-65535", "1-52,54-66,68-65535"),
        ("255.255.255.255".into(), "1-65535", "1-66,68-65535"),
    ] {
        egress.push(json!({"action":"drop","state":"enabled","destination":destination,"protocol":"tcp","destination_port":tcp}));
        egress.push(json!({"action":"drop","state":"enabled","destination":destination,"protocol":"udp","destination_port":udp}));
        egress.push(
            json!({"action":"drop","state":"enabled","destination":destination,"protocol":"icmp4"}),
        );
    }
    for protocol in ["tcp", "udp", "icmp4"] {
        egress.push(json!({"action":"allow","state":"enabled","destination":"0.0.0.0/0","protocol":protocol}));
    }
    json!({"ingress":ingress,"egress":egress})
}
fn cidrs_without(
    network: std::net::Ipv4Addr,
    prefix: u32,
    excluded: std::net::Ipv4Addr,
) -> Vec<String> {
    let mut base = u32::from(network);
    let point = u32::from(excluded);
    let mut result = Vec::new();
    for length in prefix + 1..=32 {
        let bit = 1u32 << (32 - length);
        let other = if point & bit == 0 { base | bit } else { base };
        result.push(format!("{}/{length}", std::net::Ipv4Addr::from(other)));
        if point & bit != 0 {
            base |= bit;
        }
    }
    result
}
fn verify_acl(
    acl: &Value,
    proxy: &[std::net::Ipv4Addr],
    monitor: &[std::net::Ipv4Addr],
    gateway: std::net::Ipv4Addr,
) -> Result<()> {
    let expected = expected_acl(proxy, monitor, gateway);
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
    fn raw_queries_use_the_url_project_and_regular_commands_use_the_project_flag() {
        let query = vec![
            "query".into(),
            "/1.0/networks/lkjmcbr2?project=lkjmc-tenants-v2".into(),
        ];
        assert_eq!(
            scoped_args("lkjmc-tenants-v2", &query).unwrap(),
            vec![
                "--force-local",
                "query",
                "/1.0/networks/lkjmcbr2?project=lkjmc-tenants-v2"
            ]
        );
        for url in [
            "/1.0/networks/lkjmcbr2",
            "/1.0/networks/lkjmcbr2?project=default",
            "/1.0/networks/lkjmcbr2?project=lkjmc-tenants-v2&project=default",
        ] {
            assert!(scoped_args("lkjmc-tenants-v2", &["query".into(), url.into()]).is_err());
        }
        assert_eq!(
            scoped_args("lkjmc-tenants-v2", &["list".into(), "--format=json".into()]).unwrap(),
            vec![
                "--force-local",
                "--project",
                "lkjmc-tenants-v2",
                "list",
                "--format=json"
            ]
        );
    }
    #[test]
    fn deployment_acl_matches_the_runtime_contract() {
        let inventory: Value =
            serde_json::from_str(include_str!("../../../ops/production/inventory.json")).unwrap();
        verify_acl(
            &inventory["tenant_acl"],
            &["10.203.61.10".parse().unwrap()],
            &["10.203.62.1".parse().unwrap()],
            "10.203.62.1".parse().unwrap(),
        )
        .unwrap();
    }
    #[test]
    fn private_destinations_and_ingress_cannot_be_weakened() {
        let proxy = ["10.203.61.10".parse().unwrap()];
        let monitor = ["10.203.62.1".parse().unwrap()];
        let gateway = monitor[0];
        let mut acl = expected_acl(&proxy, &monitor, gateway);
        verify_acl(&acl, &proxy, &monitor, gateway).unwrap();
        acl["egress"]
            .as_array_mut()
            .unwrap()
            .retain(|r| r["destination"] != "10.0.0.0/9");
        assert!(verify_acl(&acl, &proxy, &monitor, gateway).is_err());
        let mut acl = expected_acl(&proxy, &monitor, gateway);
        acl["ingress"][0]["source"] = serde_json::json!("0.0.0.0/0");
        assert!(verify_acl(&acl, &proxy, &monitor, gateway).is_err());
        let mut acl = expected_acl(&proxy, &monitor, gateway);
        acl["egress"][0]["state"] = serde_json::json!("disabled");
        assert!(verify_acl(&acl, &proxy, &monitor, gateway).is_err());
    }
    #[test]
    fn bridge_exception_never_exposes_another_private_address() {
        let gateway = "10.203.62.1".parse().unwrap();
        let blocks = cidrs_without("10.0.0.0".parse().unwrap(), 8, gateway);
        let contains = |point: &str| {
            let point = u32::from(point.parse::<std::net::Ipv4Addr>().unwrap());
            blocks.iter().any(|block| {
                let (base, length) = block.split_once('/').unwrap();
                let length = length.parse::<u32>().unwrap();
                let base = u32::from(base.parse::<std::net::Ipv4Addr>().unwrap());
                point >> (32 - length) == base >> (32 - length)
            })
        };
        assert!(!contains("10.203.62.1"));
        for point in [
            "10.203.62.0",
            "10.203.62.2",
            "10.203.61.2",
            "10.250.0.137",
            "10.0.0.0",
            "10.255.255.255",
        ] {
            assert!(contains(point), "private destination escaped: {point}");
        }
    }
}
