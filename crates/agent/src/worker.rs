use crate::{
    capacity::{self, MIB, Resources},
    client::{Client, file_hash},
    config::{Binding, Config},
    incus::Incus,
    probe,
    state::{Store, atomic},
};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    fs::File,
    os::unix::fs::MetadataExt,
    sync::Arc,
    time::{Duration, Instant},
};
use uuid::Uuid;

pub struct Worker {
    pub config: Config,
    pub client: Client,
    pub incus: Incus,
    pub store: Store,
}
impl Worker {
    pub fn new(config: Config) -> Result<Self> {
        if !config.development {
            for path in [&config.credential_file, &config.forwarding_secret_file] {
                crate::management::secure(path)?;
                ensure!(
                    std::fs::metadata(path)?.mode() & 0o077 == 0,
                    "Host credentials must be private to root"
                );
            }
        }
        let client = Client::new(
            &config.core_url,
            &config.credential_file,
            config.core_address,
        )?;
        let store = Store::open(&config.state_dir)?;
        let incus = Incus {
            config: config.clone(),
        };
        Ok(Self {
            config,
            client,
            incus,
            store,
        })
    }
    pub async fn run(self) -> Result<()> {
        let projection = self.client.request("/internal/v1/projection", None).await?;
        ensure!(
            projection["role"] == "host",
            "Credential is not a host credential"
        );
        let worker = Arc::new(self);
        // A VM export can take minutes. Keep readiness observations independent of jobs.
        let observer = worker.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(15));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                if let Err(e) = observer.observe().await {
                    tracing::warn!(error=%e,"Host observation failed");
                }
            }
        });
        worker.run_jobs().await
    }
    async fn run_jobs(&self) -> Result<()> {
        loop {
            let job = match self
                .client
                .request("/internal/v1/poll", Some(json!({})))
                .await
            {
                Ok(v) => v["job"].clone(),
                Err(e) => {
                    tracing::warn!(error=%e,"Core poll failed");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };
            if job.is_null() {
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
            let heartbeat_client = self.client.clone();
            let heartbeat_job = job.clone();
            let heartbeat = tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(25)).await;
                    if let Err(e) = heartbeat_client
                        .ack(&heartbeat_job, "leased", json!({}), None)
                        .await
                    {
                        tracing::warn!(error=%e,"Host job lease renewal failed");
                    }
                }
            });
            let outcome = self.execute(&job).await;
            match outcome {
                Ok(result) => {
                    if let Err(e) = self.client.ack(&job, "succeeded", result, None).await {
                        tracing::warn!(error=%e,"Stored result awaits acknowledgement");
                    }
                }
                Err(e) => {
                    let message = e.to_string();
                    let none = e
                        .downcast_ref::<crate::incus::GuestFailure>()
                        .is_some_and(|e| e.no_effect);
                    tracing::warn!(job=%job["id"],error=%message,"Host operation did not complete");
                    let _ = self
                        .client
                        .ack(
                            &job,
                            if none { "failed" } else { "waiting" },
                            if none {
                                json!({"effect":"none"})
                            } else {
                                json!({})
                            },
                            Some(&message),
                        )
                        .await;
                }
            }
            heartbeat.abort();
        }
    }
    fn lock(&self) -> Result<File> {
        crate::management::operations_lock(&self.config.operations_lock, !self.config.development)
    }
    async fn execute(&self, job: &Value) -> Result<Value> {
        let _lock = self.lock()?;
        let management = crate::management::guard(&self.config).await?;
        let id = uid(job, "id")?;
        let context = self
            .client
            .request(
                &format!("/internal/v1/jobs/{id}/context"),
                Some(json!({"lease_token":job["lease_token"]})),
            )
            .await?;
        if let Some(message) = context["rejected"].as_str() {
            return Err(crate::incus::GuestFailure {
                message: message.into(),
                no_effect: context["effect"] == "none",
            }
            .into());
        }
        let server = &context["server"];
        let server_id = uid(server, "id")?;
        ensure!(
            Some(server_id) == job["server_id"].as_str().and_then(|s| s.parse().ok()),
            "Job target mismatch"
        );
        let kind = string(job, "kind")?;
        let passive = matches!(kind, "server.logs" | "server.files" | "server.file.read");
        if passive {
            // Read jobs have no host attempt/receipt and never wake or bootstrap a VM.
            // Every replay obtains fresh authorization and fresh data.
            let read = async {
                let binding = self.binding(server_id).context(
                    "The server has not been provisioned. Files and logs are unavailable.",
                )?;
                let instance = self.incus.instance(&binding).await?;
                self.incus.verify(&binding, &instance)?;
                ensure!(
                    instance["status"] == "Running",
                    "The server VM is sleeping. Files and logs are unavailable until it is running; this read did not wake it."
                );
                let action = match kind {
                    "server.logs" => "logs",
                    "server.files" => "files",
                    _ => "file_read",
                };
                self.incus
                    .helper(&binding, action, job["payload"].clone())
                    .await
            };
            return tokio::time::timeout(Duration::from_secs(15), read)
                .await
                .map_err(|_| anyhow::anyhow!("The bounded server read timed out; retry later."))
                .and_then(|v| v)
                .map_err(|e| {
                    crate::incus::GuestFailure {
                        message: e.to_string(),
                        no_effect: true,
                    }
                    .into()
                });
        }
        // File/OP receipts are verified by the guest even when the host previously
        // committed. A saved host receipt alone cannot prove the current effect.
        if !matches!(
            kind,
            "server.install"
                | "server.file.write"
                | "server.file.delete"
                | "server.directory.create"
                | "server.operator"
        ) {
            if let Some(receipt) = self.store.read::<Value>("jobs", id)? {
                if receipt["phase"] == "committed" {
                    return Ok(receipt["result"].clone());
                }
            }
        }

        if kind == "server.create"
            && server["software"] != "custom"
            && !self
                .config
                .presets
                .iter()
                .any(|p| server["software"] == p.software && server["version"] == p.version)
            && self.store.read::<Binding>("bindings", server_id)?.is_none()
        {
            let instances = self
                .incus
                .json(
                    &self.config.tenant_project,
                    &["list".into(), "--format=json".into()],
                )
                .await?;
            ensure!(
                !instances
                    .as_array()
                    .context("Invalid Incus inventory")?
                    .iter()
                    .any(|v| v["expanded_config"]["user.lkjmc.server-id"] == server_id.to_string()),
                "An existing server VM requires reconciliation before rejecting its preset"
            );
            // Both local and daemon inventories prove absence before rejection.
            return Err(crate::incus::GuestFailure {
                message: "This server software and version are not available on the host.".into(),
                no_effect: true,
            }
            .into());
        }
        if server["kind"] == "custom" {
            self.incus.verify_network().await?;
        }
        let before = if kind == "server.create" {
            // The complete project inventory also records absence and assigned addresses.
            json!({"instances":self.incus
                .json(
                    &self.config.tenant_project,
                    &["list".into(), "--format=json".into()],
                )
                .await?, "bindings":self.store.all::<Binding>("bindings")?})
        } else {
            self.incus.instance(&self.binding(server_id)?).await?
        };
        let attempt =
            crate::management::Attempt::prepare(&self.store, management, job, server, before)
                .await?;
        let binding = if kind == "server.create" {
            self.create(server).await?
        } else {
            self.binding(server_id)?
        };
        self.incus
            .verify(&binding, &self.incus.instance(&binding).await?)?;
        let result = match kind {
            "server.create" => self.status_result(&binding, "stopped", server),
            "server.start" => {
                self.start(&binding, server).await?;
                self.status_result(&binding, "running", server)
            }
            "server.stop" => {
                self.stop(&binding).await?;
                self.status_result(&binding, "stopped", server)
            }
            "server.file.write"
            | "server.file.delete"
            | "server.directory.create"
            | "server.operator" => {
                ensure!(
                    binding.custom
                        && server["desired"] == "stopped"
                        && server["observed"] == "stopped",
                    "Stop the custom game before changing files or Minecraft OP."
                );
                let result = async {
                    self.ensure_guest(&binding, server).await?;
                    if kind == "server.operator" {
                        let fresh = self
                            .client
                            .request(
                                &format!("/internal/v1/jobs/{id}/context"),
                                Some(json!({"lease_token":job["lease_token"]})),
                            )
                            .await?;
                        if let Some(message) = fresh["rejected"].as_str() {
                            return Err(crate::incus::GuestFailure {
                                message: message.into(),
                                no_effect: fresh["effect"] == "none",
                            }
                            .into());
                        }
                    }
                    let action = match kind {
                        "server.file.write" => "file_write",
                        "server.file.delete" => "file_delete",
                        "server.directory.create" => "directory_create",
                        _ => "operator",
                    };
                    let mut payload = job["payload"].clone();
                    payload["job_id"] = json!(id);
                    let result = self.incus.helper(&binding, action, payload).await?;
                    ensure!(
                        result["effect"] == "committed",
                        "The guest has not verified the filesystem effect."
                    );
                    Ok::<Value, anyhow::Error>(result)
                }
                .await;
                self.incus.power(&binding, false).await?;
                result?
            }
            "server.console" => {
                ensure!(
                    self.incus.instance(&binding).await?["status"] == "Running",
                    "サーバーが停止しています。"
                );
                let result = self
                    .incus
                    .helper(
                        &binding,
                        "console",
                        json!({"job_id":id,"line":job["payload"]["line"]}),
                    )
                    .await?;
                ensure!(
                    result["effect"] == "committed",
                    "コンソール送信の保存境界で中断しました。二重実行を避けるため、ログを確認してから別の操作として送信してください。"
                );
                result
            }
            "server.install" => {
                ensure!(
                    binding.custom
                        && server["desired"] == "stopped"
                        && server["observed"] == "stopped",
                    "停止済みの個人サーバーだけにファイルを反映できます。"
                );
                let a = &context["artifact"];
                let artifact = uid(a, "id")?;
                let path = self.store.root.join("downloads").join(artifact.to_string());
                self.download(
                    &format!("/internal/v1/artifacts/{artifact}"),
                    true,
                    string(a, "sha256")?,
                    a["bytes"].as_u64().context("Missing artifact size")?,
                    &path,
                )
                .await?;
                let result = async {
                self.ensure_guest(&binding, server).await?;
                self.incus
                    .push(
                        &binding,
                        &path,
                        &format!("/var/lib/lkjmc/incoming/{artifact}"),
                    )
                    .await?;
                self.incus.helper(&binding,"install",json!({"job_id":id,"artifact_id":artifact,"sha256":a["sha256"],"kind":a["kind"],"path":job["payload"]["path"],"storage_mib":server["storage_mib"]})).await
                }.await;
                self.incus.power(&binding, false).await?;
                let result = result?;
                ensure!(
                    result["effect"] == "committed"
                        && result["path"] == job["payload"]["path"]
                        && result["sha256"] == a["sha256"],
                    "The guest artifact receipt does not match the requested exact destination and digest."
                );
                result
            }
            "server.backup" => self.backup(job, &binding, server).await?,
            "server.restore" => {
                self.restore(job, &binding, server, &context["backup"])
                    .await?
            }
            "official.backup" => self.official_backup(job, &binding, server).await?,
            "official.backup.prune" => self.prune_backup(job, &binding, &context["backup"]).await?,
            _ => anyhow::bail!("Unknown host operation; no implicit success"),
        };
        attempt.finish(&self.store, &result).await?;
        self.store
            .write("jobs", id, &json!({"phase":"committed","result":result}))?;
        Ok(result)
    }
    fn binding(&self, id: Uuid) -> Result<Binding> {
        if let Some(binding) = self.config.trusted_servers.get(&id) {
            return Ok(binding.clone());
        }
        self.store
            .read("bindings", id)?
            .context("No managed VM binding exists for this server")
    }
    async fn create(&self, server: &Value) -> Result<Binding> {
        ensure!(
            server["kind"] == "custom",
            "Host creation is limited to tenant VMs"
        );
        let id = uid(server, "id")?;
        ensure!(
            self.config.development
                || server["storage_mib"]
                    .as_i64()
                    .is_some_and(|size| size >= crate::hosting_limits::MIN_SERVER_STORAGE_MIB),
            "Servers need at least 16384 MiB of storage."
        );
        self.capacity(server, true).await?;
        let preset = self
            .config
            .presets
            .iter()
            .find(|p| server["software"] == p.software && server["version"] == p.version);
        ensure!(
            preset.is_some() || server["software"] == "custom",
            "このバージョンのプリセットをホストに準備する必要があります。"
        );
        let binding = if let Some(existing) = self.store.read::<Binding>("bindings", id)? {
            existing
        } else {
            let bindings = self.store.all::<Binding>("bindings")?;
            let address = self
                .config
                .addresses
                .iter()
                .find(|address| !bindings.iter().any(|b| b.address == **address))
                .context("VMアドレスの割り当て枠がいっぱいです。")?;
            let binding = Binding {
                server_id: id,
                project: self.config.tenant_project.clone(),
                instance: format!("lkjmc-server-{id}"),
                address: *address,
                custom: true,
            };
            self.store.write("bindings", id, &binding)?;
            binding
        };
        // Inspect the complete list so that a transport failure cannot be mistaken for absence.
        let all = self
            .incus
            .json(&binding.project, &["list".into(), "--format=json".into()])
            .await?;
        if !all
            .as_array()
            .context("Invalid instance inventory")?
            .iter()
            .any(|v| v["name"] == binding.instance)
        {
            let cpu = Resources::requested(server)?.cpu;
            let args = vec![
                "init".into(),
                self.config.image_fingerprint.clone(),
                binding.instance.clone(),
                "--vm".into(),
                "--profile".into(),
                self.config.tenant_profile.clone(),
                "--storage".into(),
                self.config.storage_pool.clone(),
                "--config".into(),
                format!("limits.memory={}MiB", server["memory_mib"]),
                "--config".into(),
                format!("limits.cpu={cpu}"),
                "--config".into(),
                format!("user.lkjmc.server-id={id}"),
                "--config".into(),
                "user.lkjmc.scope=rebuild".into(),
                "--config".into(),
                "boot.autostart=false".into(),
                "--device".into(),
                format!("root,size={}MiB", server["storage_mib"]),
                "--device".into(),
                format!("eth0,ipv4.address={}", binding.address),
                "--device".into(),
                "eth0,security.mac_filtering=true".into(),
                "--device".into(),
                "eth0,security.ipv4_filtering=true".into(),
                "--device".into(),
                "eth0,security.ipv6_filtering=true".into(),
                "--device".into(),
                "eth0,security.port_isolation=true".into(),
            ];
            self.incus.run(&binding.project, &args, None).await?;
        }
        self.ensure_guest(&binding, server).await?;
        let helper = self.store.root.join("guest.py");
        atomic(&helper, include_bytes!("../../../ops/guest/guest.py"))?;
        self.incus
            .push(&binding, &helper, "/usr/local/lib/lkjmc/guest.py")
            .await?;
        let secret = std::fs::read_to_string(&self.config.forwarding_secret_file)?;
        self.incus.helper(&binding,"bootstrap",json!({"server_id":id,"software":server["software"],"memory_mib":server["memory_mib"],"java":preset.map(|p|p.java).unwrap_or(25),"forwarding_secret":secret.trim()})).await?;
        if let Some(preset) = preset {
            let artifact = Uuid::from_u128(id.as_u128() ^ 0x314d35e211064a349f28104baeee0101);
            let file = self.store.root.join("downloads").join(artifact.to_string());
            self.download(
                &preset.url,
                false,
                &preset.sha256,
                1024 * 1024 * 1024,
                &file,
            )
            .await?;
            self.incus
                .push(
                    &binding,
                    &file,
                    &format!("/var/lib/lkjmc/incoming/{artifact}"),
                )
                .await?;
            self.incus.helper(&binding,"install",json!({"job_id":artifact,"artifact_id":artifact,"sha256":preset.sha256,"kind":"jar","path":"server.jar","storage_mib":server["storage_mib"]})).await?;
        }
        self.incus.power(&binding, false).await?;
        Ok(binding)
    }
    async fn capacity(&self, server: &Value, creating: bool) -> Result<()> {
        let requested = Resources::requested(server)?;
        let mut observed = None;
        if server["kind"] == "custom" {
            let projection = self.client.request("/internal/v1/projection", None).await?;
            let servers = projection["servers"]
                .as_array()
                .context("Missing server registry")?;
            let instances = self
                .incus
                .json(
                    &self.config.tenant_project,
                    &["list".into(), "--format=json".into()],
                )
                .await?;
            let instances = instances
                .as_array()
                .context("Missing tenant VM inventory")?;
            let bindings = self.store.all::<Binding>("bindings")?;
            for instance in instances {
                let id: Uuid = instance["expanded_config"]["user.lkjmc.server-id"]
                    .as_str()
                    .context("Unidentified tenant VM; reconcile before allocating")?
                    .parse()?;
                let binding = bindings
                    .iter()
                    .find(|b| b.server_id == id)
                    .context("Unbound tenant VM; reconcile before allocating")?;
                self.incus.verify(binding, instance)?;
                if instance["expanded_config"]["user.lkjmc.server-id"] == server["id"] {
                    observed = Some(instance.clone());
                }
            }
            let usage = capacity::tenant_usage(
                servers,
                instances,
                &bindings.iter().map(|b| b.server_id).collect(),
                server,
            )?;
            ensure!(
                usage.storage <= u128::from(self.config.max_tenant_storage_mib) * u128::from(MIB),
                "ホストで割り当て可能な保存容量を超えます。"
            );
            ensure!(
                usage.memory <= u128::from(self.config.max_tenant_memory_mib) * u128::from(MIB),
                "ホストの同時稼働用メモリを超えます。"
            );
            ensure!(
                usage.cpu <= u128::from(self.config.max_tenant_cpu),
                "ホストの同時稼働用CPU枠を超えます。"
            );
        } else {
            let binding = self.binding(uid(server, "id")?)?;
            let instance = self.incus.instance(&binding).await?;
            self.incus.verify(&binding, &instance)?;
            observed = Some(instance);
        }
        let allocated = observed.as_ref().map(Resources::instance).transpose()?;
        if observed.as_ref().is_none_or(|i| i["status"] == "Stopped") {
            let memory = requested
                .memory
                .max(allocated.map(|r| r.memory).unwrap_or(0));
            ensure!(
                capacity::available_memory()?
                    >= memory + u128::from(self.config.host_memory_reserve_mib) * u128::from(MIB),
                "実測した空きメモリが不足しています。既存サーバー用の余裕を残して起動を待機します。"
            );
        }
        capacity::preserve_pool(
            &self.config.storage_pool_path,
            self.config.pool_free_reserve_mib,
            if creating && observed.is_none() {
                requested.storage
            } else {
                0
            },
        )?;
        capacity::archive_available(&self.store.root, self.config.max_archive_mib)?;
        Ok(())
    }
    async fn ensure_guest(&self, b: &Binding, server: &Value) -> Result<()> {
        // Includes temporary boots for mutations and resumes after backups.
        self.capacity(server, false).await?;
        self.incus.power(b, true).await?;
        let deadline = Instant::now() + Duration::from_secs(120);
        loop {
            match self.incus.exec(b, &["/usr/bin/true".into()], None).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if Instant::now() > deadline {
                        return Err(e).context("VM agent did not become ready");
                    }
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }
    async fn start(&self, b: &Binding, server: &Value) -> Result<()> {
        self.ensure_guest(b, server).await?;
        self.incus.helper(b, "start", json!({})).await?;
        let deadline = Instant::now() + Duration::from_secs(120);
        loop {
            if probe::minecraft(b.address).await.is_ok() {
                if b.custom {
                    return Ok(());
                }
                let projection = self.client.request("/internal/v1/projection", None).await?;
                if projection["servers"]
                    .as_array()
                    .context("Missing registry")?
                    .iter()
                    .any(|s| {
                        s["id"] == server["id"]
                            && s["observed"] == "running"
                            && s["capabilities"]["adapter_ready"] == true
                    })
                {
                    return Ok(());
                }
            }
            ensure!(
                Instant::now() < deadline,
                "Minecraftの起動確認を待っています。コンソールログを確認してください。"
            );
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
    async fn stop(&self, b: &Binding) -> Result<()> {
        let instance = self.incus.instance(b).await?;
        self.incus.verify(b, &instance)?;
        if instance["status"] == "Stopped" {
            return Ok(());
        }
        self.incus.helper(b, "stop", json!({})).await?;
        self.incus.power(b, false).await?;
        Ok(())
    }
    fn status_result(&self, b: &Binding, state: &str, server: &Value) -> Value {
        let mut result =
            json!({"effect":"committed","observed":state,"address":format!("{}:25565",b.address)});
        if b.custom {
            let paper = server["software"] == "paper";
            result["capabilities"] = json!({"proxy_join":paper,"vanilla_client":paper,"bedrock":paper,"official_progression":false,"isolated_vm":true,"java":self.config.presets.iter().find(|p|server["software"]==p.software&&server["version"]==p.version).map(|p|p.java).unwrap_or(25)});
        }
        result
    }
    async fn backup(&self, job: &Value, b: &Binding, server: &Value) -> Result<Value> {
        ensure!(
            b.custom,
            "Trusted official state requires the official backup barrier"
        );
        let backup = uid(&job["payload"], "backup_id")?;
        let checkpoint = self
            .store
            .root
            .join("backups")
            .join(format!("{backup}.json"));
        if checkpoint.exists() {
            let previous: Value = serde_json::from_slice(&std::fs::read(&checkpoint)?)?;
            if previous.get("result").is_some() {
                if previous["resume"] == true && self.should_resume(b.server_id).await? {
                    self.start(b, server).await?;
                }
                return Ok(previous["result"].clone());
            }
        }
        let resume = if checkpoint.exists() {
            serde_json::from_slice::<Value>(&std::fs::read(&checkpoint)?)?["resume"]
                .as_bool()
                .unwrap_or(false)
        } else {
            let value = server["desired"] == "running";
            atomic(&checkpoint, &serde_json::to_vec(&json!({"resume":value}))?)?;
            value
        };
        self.stop(b).await?;
        let result = self.export(b, backup).await?;
        atomic(
            &checkpoint,
            &serde_json::to_vec(&json!({"resume":resume,"result":result}))?,
        )?;
        if resume && self.should_resume(b.server_id).await? {
            self.start(b, server).await?;
        }
        Ok(result)
    }
    async fn download(
        &self,
        url: &str,
        internal: bool,
        expected: &str,
        limit: u64,
        path: &std::path::Path,
    ) -> Result<()> {
        if path.exists() && file_hash(path).await? == expected {
            return Ok(());
        }
        // A failed transfer is not a completed backup. Discard only its known
        // temporary path; retain every verified artifact and its receipt.
        let temporary = path.with_extension("part");
        if temporary.exists() {
            std::fs::remove_file(&temporary)?;
        }
        ensure!(
            limit <= capacity::archive_available(&self.store.root, self.config.max_archive_mib)?,
            "ダウンロードを保存する容量が不足しています。"
        );
        capacity::preserve_pool(
            &self.config.storage_pool_path,
            self.config.pool_free_reserve_mib,
            u128::from(limit),
        )?;
        self.client
            .download(url, internal, expected, limit, path)
            .await
    }
    async fn export(&self, b: &Binding, backup: Uuid) -> Result<Value> {
        ensure!(
            self.incus.instance(b).await?["status"] == "Stopped",
            "VM must remain stopped throughout export"
        );
        let snapshot = format!("b-{backup}");
        let snapshots = self
            .incus
            .json(
                &b.project,
                &[
                    "snapshot".into(),
                    "list".into(),
                    b.instance.clone(),
                    "--format=json".into(),
                ],
            )
            .await?;
        if !snapshots
            .as_array()
            .context("Invalid snapshot list")?
            .iter()
            .any(|s| {
                s["name"]
                    .as_str()
                    .is_some_and(|n| n == snapshot || n.ends_with(&format!("/{snapshot}")))
            })
        {
            self.incus
                .run(
                    &b.project,
                    &[
                        "snapshot".into(),
                        "create".into(),
                        b.instance.clone(),
                        snapshot.clone(),
                    ],
                    None,
                )
                .await?;
        }
        let archive = self
            .store
            .root
            .join("backups")
            .join(format!("{backup}.tar.gz"));
        if !archive.exists() {
            let temporary = archive.with_extension("partial");
            if temporary.exists() {
                std::fs::remove_file(&temporary)?;
            }
            let disk = Resources::instance(&self.incus.instance(b).await?)?.storage;
            // Incus may stage an export on the pool before streaming to us.
            // Reserve one full disk plus metadata for each of both copies.
            capacity::preserve_pool(
                &self.config.storage_pool_path,
                self.config.pool_free_reserve_mib,
                2 * (disk + 1024 * u128::from(MIB)),
            )?;
            let limit = capacity::archive_available(&self.store.root, self.config.max_archive_mib)?;
            self.incus.export(b, &temporary, limit).await?;
            File::open(&temporary)?.sync_all()?;
            std::fs::rename(temporary, &archive)?;
            File::open(archive.parent().unwrap())?.sync_all()?;
        }
        let result = json!({"effect":"committed","verified":true,"backup_id":backup,"server_id":b.server_id,"snapshot":snapshot,"sha256":file_hash(&archive).await?,"bytes":std::fs::metadata(&archive)?.len()});
        Ok(result)
    }
    async fn official_backup(&self, job: &Value, b: &Binding, server: &Value) -> Result<Value> {
        ensure!(
            !b.custom && server["kind"] == "official",
            "Only the trusted official instance can enter the official backup barrier"
        );
        let id = uid(job, "id")?;
        let backup = uid(&job["payload"], "backup_id")?;
        let path = self
            .store
            .root
            .join("backups")
            .join(format!("{backup}.json"));
        let mut checkpoint = if path.exists() {
            serde_json::from_slice::<Value>(&std::fs::read(&path)?)?
        } else {
            json!({"resume":server["desired"]=="running"})
        };
        atomic(&path, &serde_json::to_vec(&checkpoint)?)?;
        let route = format!("/internal/v1/jobs/{id}/backup");
        if checkpoint.get("result").is_none() {
            self.stop(b).await?;
            self.client.request("/internal/v1/observations",Some(json!({"server_id":b.server_id,"observed":"stopped","players":0,"metrics":{"backup_id":backup}}))).await?;
            let mut step = self
                .client
                .request(
                    &route,
                    Some(json!({"lease_token":job["lease_token"],"action":"freeze"})),
                )
                .await?;
            ensure!(
                step["phase"] != "released",
                "Core already released this backup but the host archive receipt is absent; reconcile the stored files"
            );
            while step["phase"] != "dumped" {
                step = self
                    .client
                    .request(
                        &route,
                        Some(json!({"lease_token":job["lease_token"],"action":"dump"})),
                    )
                    .await?;
                if let Some(error) = step["error"].as_str() {
                    anyhow::bail!("DB保存を回復中です: {error}");
                }
                if step["phase"] != "dumped" {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
            let database = step["database_manifest"].clone();
            self.download(
                &format!("/internal/v1/official-backups/{backup}/database"),
                true,
                string(&database, "sha256")?,
                database["bytes"]
                    .as_u64()
                    .context("Missing database size")?,
                &self
                    .store
                    .root
                    .join("backups")
                    .join(format!("{backup}.dump")),
            )
            .await?;
            let world = self.export(b, backup).await?;
            checkpoint["result"] = json!({"effect":"committed","verified":true,"backup_id":backup,"database":database,"world":world,"consistency":"stopped-world-and-frozen-official-state","restore_tested":false});
            // Both artifacts and this receipt are durable before Core can unfreeze.
            atomic(&path, &serde_json::to_vec(&checkpoint)?)?;
        }
        let result = checkpoint["result"].clone();
        self.client.request(&route,Some(json!({"lease_token":job["lease_token"],"action":"release","world_manifest":result["world"]}))).await?;
        if checkpoint["resume"] == true && self.should_resume(b.server_id).await? {
            self.start(b, server).await?;
        }
        Ok(result)
    }
    async fn should_resume(&self, id: Uuid) -> Result<bool> {
        let projection = self.client.request("/internal/v1/projection", None).await?;
        Ok(projection["servers"]
            .as_array()
            .context("Missing registry")?
            .iter()
            .any(|s| s["id"] == id.to_string() && s["desired"] == "running"))
    }
    async fn prune_backup(&self, job: &Value, b: &Binding, backup: &Value) -> Result<Value> {
        ensure!(
            !b.custom
                && backup["kind"] == "official"
                && backup["state"] == "pruning"
                && backup["pinned"] == false,
            "Only an authorized official backup generation can be removed"
        );
        let id = uid(backup, "id")?;
        let job_id = uid(job, "id")?;
        ensure!(
            backup["prune_job_id"] == job["id"] && backup["server_id"] == b.server_id.to_string(),
            "Backup retention target mismatch"
        );
        let snapshot = format!("b-{id}");
        let snapshots = self
            .incus
            .json(
                &b.project,
                &[
                    "snapshot".into(),
                    "list".into(),
                    b.instance.clone(),
                    "--format=json".into(),
                ],
            )
            .await?;
        let exists = snapshots
            .as_array()
            .context("Invalid snapshot list")?
            .iter()
            .any(|s| {
                s["name"]
                    .as_str()
                    .is_some_and(|n| n == snapshot || n == format!("{}/{snapshot}", b.instance))
            });
        let manifest = &backup["manifest"];
        ensure!(
            exists || crate::retention::intent(&self.store, id, job_id, b.server_id, manifest)?,
            "Snapshot is absent before deletion was prepared; reconcile this backup"
        );
        crate::retention::prepare(&self.store, id, job_id, b.server_id, manifest).await?;
        if exists {
            self.incus
                .run(
                    &b.project,
                    &[
                        "snapshot".into(),
                        "delete".into(),
                        b.instance.clone(),
                        snapshot,
                    ],
                    None,
                )
                .await?;
        }
        crate::retention::remove_files(&self.store, id, job_id, b.server_id, manifest)?;
        let receipt = self
            .client
            .request(
                &format!("/internal/v1/jobs/{job_id}/backup-prune"),
                Some(json!({"lease_token":job["lease_token"]})),
            )
            .await?;
        ensure!(
            receipt["database_deleted"] == true && receipt["backup_id"] == id.to_string(),
            "Core database copy still awaits deletion"
        );
        Ok(
            json!({"effect":"committed","backup_id":id,"server_id":b.server_id,"host_deleted":true,"database_deleted":true}),
        )
    }
    async fn restore(
        &self,
        job: &Value,
        b: &Binding,
        server: &Value,
        backup: &Value,
    ) -> Result<Value> {
        ensure!(
            b.custom && server["desired"] == "stopped" && server["observed"] == "stopped",
            "停止済みの個人サーバーだけを復元できます。"
        );
        let id = uid(&job["payload"], "backup_id")?;
        let archive = self.store.root.join("backups").join(format!("{id}.tar.gz"));
        ensure!(
            backup["manifest"]["server_id"] == b.server_id.to_string()
                && file_hash(&archive).await? == string(&backup["manifest"], "sha256")?,
            "Backup identity or hash does not match"
        );
        self.stop(b).await?;
        self.incus
            .run(
                &b.project,
                &[
                    "snapshot".into(),
                    "restore".into(),
                    b.instance.clone(),
                    format!("b-{id}"),
                ],
                None,
            )
            .await?;
        self.incus.verify(b, &self.incus.instance(b).await?)?;
        Ok(self.status_result(b, "stopped", server))
    }
    pub async fn observe(&self) -> Result<()> {
        let mut bindings = self.store.all::<Binding>("bindings")?;
        bindings.extend(self.config.trusted_servers.values().cloned());
        for b in bindings {
            let instance = match self.incus.instance(&b).await {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!(server=%b.server_id,error=%e,"Cannot observe instance");
                    continue;
                }
            };
            if let Err(e) = self.incus.verify(&b, &instance) {
                tracing::warn!(server=%b.server_id,error=%e,"Instance isolation verification failed");
                continue;
            }
            let running = instance["status"] == "Running";
            if !b.custom && running {
                continue;
            } // The trusted Paper adapter owns readiness and player counts.
            let ping = if running {
                probe::minecraft(b.address).await.ok()
            } else {
                None
            };
            let players = ping
                .as_ref()
                .and_then(|p| p["players"]["online"].as_i64())
                .unwrap_or(0)
                .clamp(0, 1_000_000);
            self.client.request("/internal/v1/observations",Some(json!({"server_id":b.server_id,"observed":if ping.is_some(){"running"}else if running{"starting"}else{"stopped"},"players":players,"metrics":{"instance":b.instance,"isolated_vm":b.custom,"minecraft_status":ping,"memory":instance["state"]["memory"],"cpu":instance["state"]["cpu"]}}))).await?;
        }
        Ok(())
    }
}
fn uid(value: &Value, key: &str) -> Result<Uuid> {
    Ok(string(value, key)?.parse()?)
}
fn string<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key]
        .as_str()
        .with_context(|| format!("Missing {key}"))
}
