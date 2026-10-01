use crate::{
    client::{Client, file_hash},
    config::{Binding, Config},
    incus::Incus,
    probe,
    state::{Store, atomic},
};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    fs::{File, OpenOptions},
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
        let client = Client::new(&config.core_url, &config.credential_file)?;
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
        // Same host-wide operations lock used by the authoritative infrastructure workflow.
        // It must already exist: this process must never invent a second lock domain.
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.config.operations_lock)
            .context("Managed operations lock is unavailable")?;
        file.try_lock()
            .map_err(|_| anyhow::anyhow!("別のホスト保守が進行中です。完了を待っています。"))?;
        Ok(file)
    }
    async fn execute(&self, job: &Value) -> Result<Value> {
        let _lock = self.lock()?;
        let id = uid(job, "id")?;
        if let Some(receipt) = self.store.read::<Value>("jobs", id)? {
            if receipt["phase"] == "committed" {
                return Ok(receipt["result"].clone());
            }
        }
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
                no_effect: true,
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
        if server["kind"] == "custom" {
            self.incus.verify_network().await?;
        }
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
                self.capacity(server, false).await?;
                self.start(&binding, server).await?;
                self.status_result(&binding, "running", server)
            }
            "server.stop" => {
                self.stop(&binding).await?;
                self.status_result(&binding, "stopped", server)
            }
            "server.logs" => {
                let stopped = self.incus.instance(&binding).await?["status"] == "Stopped";
                self.ensure_guest(&binding).await?;
                let logs = self.incus.helper(&binding, "logs", json!({})).await;
                if stopped {
                    self.incus.power(&binding, false).await?;
                }
                logs?
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
                self.client
                    .download(
                        &format!("/internal/v1/artifacts/{artifact}"),
                        true,
                        string(a, "sha256")?,
                        a["bytes"].as_u64().context("Missing artifact size")?,
                        &path,
                    )
                    .await?;
                let result = async {
                self.ensure_guest(&binding).await?;
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
                result?
            }
            "server.backup" => self.backup(job, &binding, server).await?,
            "server.restore" => {
                self.restore(job, &binding, server, &context["backup"])
                    .await?
            }
            "official.backup" => self.official_backup(job, &binding, server).await?,
            _ => anyhow::bail!("Unknown host operation; no implicit success"),
        };
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
                instance: format!("lkjmc-v2-{id}"),
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
            let cpu = (server["cpu_millis"].as_u64().context("Missing CPU limit")? + 999) / 1000;
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
                format!(
                    "eth0,ipv4.address={},security.mac_filtering=true,security.ipv4_filtering=true,security.ipv6_filtering=true,security.port_isolation=true",
                    binding.address
                ),
            ];
            self.incus.run(&binding.project, &args, None).await?;
        }
        self.ensure_guest(&binding).await?;
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
            self.client
                .download(
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
        if server["kind"] != "custom" {
            return Ok(());
        }
        let projection = self.client.request("/internal/v1/projection", None).await?;
        let servers = projection["servers"]
            .as_array()
            .context("Missing server registry")?;
        let bindings = self.store.all::<Binding>("bindings")?;
        let id = uid(server, "id")?;
        let storage: u64 = servers
            .iter()
            .filter(|s| {
                s["kind"] == "custom"
                    && (s["id"] == server["id"]
                        || bindings.iter().any(|b| {
                            Some(b.server_id) == s["id"].as_str().and_then(|x| x.parse().ok())
                        }))
            })
            .map(|s| s["storage_mib"].as_u64().unwrap_or(u64::MAX / 4096))
            .sum();
        ensure!(
            storage <= self.config.max_tenant_storage_mib,
            "ホストで割り当て可能な保存容量を超えます。管理者が実容量と使用量を確認する必要があります。"
        );
        if !creating {
            let memory: u64 = servers
                .iter()
                .filter(|s| {
                    s["kind"] == "custom" && (s["id"] == server["id"] || s["desired"] == "running")
                })
                .map(|s| s["memory_mib"].as_u64().unwrap_or(u64::MAX / 4096))
                .sum();
            ensure!(
                memory <= self.config.max_tenant_memory_mib,
                "ホストの同時稼働用メモリを超えます。"
            );
            if let Ok(binding) = self.binding(id) {
                if self.incus.instance(&binding).await?["status"] == "Running" {
                    return Ok(());
                }
            }
        }
        let memory = std::fs::read_to_string("/proc/meminfo")?;
        let available = memory
            .lines()
            .find_map(|line| {
                line.strip_prefix("MemAvailable:")
                    .and_then(|v| v.split_whitespace().next())
                    .and_then(|v| v.parse::<u64>().ok())
            })
            .context("Cannot measure available host RAM")?
            / 1024;
        ensure!(
            available
                >= server["memory_mib"].as_u64().context("Missing RAM limit")?
                    + self.config.host_memory_reserve_mib,
            "実測した空きメモリが不足しています。既存の稼働を保ったまま、確保できる容量を確認してください。"
        );
        Ok(())
    }
    async fn ensure_guest(&self, b: &Binding) -> Result<()> {
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
        self.ensure_guest(b).await?;
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
            self.incus
                .run(
                    &b.project,
                    &[
                        "export".into(),
                        b.instance.clone(),
                        temporary.to_string_lossy().into_owned(),
                        "--instance-only".into(),
                    ],
                    None,
                )
                .await?;
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
            self.client
                .download(
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
