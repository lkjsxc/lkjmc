//! Management receipts and immutable per-attempt plans, under operations.lock.
use crate::{client::file_hash, config::Config, state::Store};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
    time::SystemTime,
};
use tokio::process::Command;
use uuid::Uuid;

const REPO: &str = "/root/forgejo-restore-20260906T232532/repos/bootstrap-infra";
const CAMPAIGN: &str = "/root/forgejo-restore-20260906T232532";
const POOL: &str = "/var/lib/incus/storage-pools/default";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub schema: u32,
    pub commit: String,
    pub release_commit: String,
    pub agent_sha256: String,
    pub state_sha256: String,
    pub sources: BTreeMap<String, String>,
    pub configuration: Value,
}

pub(crate) fn secure(path: &Path) -> Result<()> {
    let info = std::fs::symlink_metadata(path)?;
    ensure!(
        info.is_file() && info.nlink() == 1 && info.uid() == 0 && info.mode() & 0o022 == 0,
        "Management evidence must be a root-owned regular file"
    );
    for parent in path.ancestors().skip(1) {
        let info = std::fs::symlink_metadata(parent)?;
        ensure!(
            info.is_dir() && info.uid() == 0 && info.mode() & 0o022 == 0,
            "Unsafe management evidence directory"
        );
    }
    Ok(())
}

pub(crate) fn operations_lock(path: &Path, production: bool) -> Result<File> {
    // File::try_lock uses flock on this Linux target, matching manage.py.
    // The existing lock must never be replaced or followed through a symlink.
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .context("Managed operations lock is unavailable")?;
    let metadata = file.metadata()?;
    ensure!(
        metadata.is_file() && metadata.nlink() == 1 && metadata.mode() & 0o077 == 0,
        "Unsafe managed operations lock"
    );
    if production {
        ensure!(
            metadata.uid() == 0,
            "Managed operations lock must be owned by root"
        );
        secure(path)?;
    }
    file.try_lock()
        .map_err(|_| anyhow::anyhow!("別のホスト保守が進行中です。完了を待っています。"))?;
    Ok(file)
}

async fn output(binary: &str, args: &[&str]) -> Result<String> {
    let out = Command::new(binary).args(args).output().await?;
    ensure!(
        out.status.success(),
        "Host management identity query failed"
    );
    Ok(String::from_utf8(out.stdout)?.trim().to_owned())
}

pub async fn guard(config: &Config) -> Result<Value> {
    if config.development {
        return Ok(json!({"development":true}));
    }
    ensure!(
        std::fs::read_to_string("/proc/sys/kernel/hostname")?.trim() == "archserver",
        "Wrong management host"
    );
    ensure!(
        output("/usr/bin/id", &["-u"]).await? == "0",
        "Management agent must run as root"
    );
    ensure!(
        output("/usr/bin/findmnt", &["-nro", "UUID", "--target", "/"]).await?
            == "ff870ada-399f-48af-a72f-eba83c50aea0",
        "Root UUID changed"
    );
    ensure!(
        output("/usr/bin/findmnt", &["-nro", "UUID", "--mountpoint", POOL]).await?
            == "68bc4d9d-fffc-471c-b276-f69f73eb84a9",
        "Pool UUID changed"
    );
    let manifest = config
        .management_manifest
        .as_ref()
        .context("Missing management receipt")?;
    secure(manifest)?;
    let evidence: Receipt = serde_json::from_slice(&std::fs::read(manifest)?)?;
    ensure!(
        evidence.schema == 1 && evidence.commit.len() == 40 && evidence.release_commit.len() == 40,
        "Unknown management receipt"
    );
    ensure!(
        serde_json::to_value(config)? == evidence.configuration,
        "Runtime configuration differs from the approved deployment"
    );
    let git = [
        "-c",
        "core.hooksPath=/dev/null",
        "-c",
        "core.fsmonitor=false",
        "-C",
        REPO,
    ];
    ensure!(
        output(
            "/usr/bin/git",
            &[git.as_slice(), &["status", "--porcelain"]].concat()
        )
        .await?
        .is_empty(),
        "Host definitions have unreviewed changes"
    );
    ensure!(
        output(
            "/usr/bin/git",
            &[git.as_slice(), &["rev-parse", "HEAD"]].concat()
        )
        .await?
            == evidence.commit,
        "Host definition revision changed; refresh the deployment receipt through management"
    );
    let state = Path::new(CAMPAIGN).join("states/lkjmc_v2.tfstate");
    secure(&state)?;
    ensure!(
        file_hash(&state).await? == evidence.state_sha256,
        "Canonical rebuild state changed"
    );
    ensure!(
        file_hash(Path::new("/proc/self/exe")).await? == evidence.agent_sha256,
        "Running agent differs from the approved release"
    );
    ensure!(
        evidence.sources.contains_key("ops/manage.py")
            && evidence.sources.contains_key("infra/lkjmc_v2/main.tf"),
        "Incomplete source receipt"
    );
    for (name, sha) in &evidence.sources {
        ensure!(
            !Path::new(name).is_absolute()
                && Path::new(name)
                    .components()
                    .all(|c| matches!(c, std::path::Component::Normal(_))),
            "Invalid source path"
        );
        let source = Path::new(REPO).join(name);
        secure(&source)?;
        ensure!(
            file_hash(&source).await? == *sha,
            "Host definition differs from its saved receipt"
        );
    }
    let pending = Path::new(CAMPAIGN).join("private/manage-pending-checkpoint.json");
    if pending.exists() {
        secure(&pending)?;
        let value: Value = serde_json::from_slice(&std::fs::read(pending)?)?;
        ensure!(
            value["acknowledged"] == true,
            "Host apply checkpoint is still pending"
        );
    }
    Ok(serde_json::to_value(evidence)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shares_the_python_management_lock_and_rejects_indirections() {
        use std::{
            io::{BufRead, BufReader, Write},
            os::unix::fs::symlink,
            process::Stdio,
        };
        let directory = std::env::temp_dir().join(format!("lkjmc-lock-{}", Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("operations.lock");
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
            .unwrap();
        let mut child = std::process::Command::new("python3")
            .args(["-c", "import fcntl,sys; f=open(sys.argv[1],'r+'); fcntl.flock(f,fcntl.LOCK_EX); print('locked',flush=True); sys.stdin.read(1)"])
            .arg(&path).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
        let mut line = String::new();
        BufReader::new(child.stdout.take().unwrap())
            .read_line(&mut line)
            .unwrap();
        assert_eq!(line.trim(), "locked");
        assert!(operations_lock(&path, false).is_err());
        child.stdin.take().unwrap().write_all(b"x").unwrap();
        assert!(child.wait().unwrap().success());
        drop(operations_lock(&path, false).unwrap());
        let link = directory.join("symlink");
        symlink(&path, &link).unwrap();
        assert!(operations_lock(&link, false).is_err());
        std::fs::hard_link(&path, directory.join("hardlink")).unwrap();
        assert!(operations_lock(&path, false).is_err());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[tokio::test]
    async fn runtime_receipt_refuses_a_changed_saved_plan() {
        let directory = std::env::temp_dir().join(format!("lkjmc-runtime-plan-{}", Uuid::new_v4()));
        let store = Store::open(&directory).unwrap();
        let job = json!({"id":Uuid::new_v4(),"kind":"server.stop","payload":{}});
        let server = json!({"id":Uuid::new_v4()});
        let attempt = Attempt::prepare(
            &store,
            json!({"commit":"reviewed"}),
            &job,
            &server,
            json!({"status":"Running"}),
        )
        .await
        .unwrap();
        attempt
            .finish(&store, &json!({"stopped":true}))
            .await
            .unwrap();
        let path = directory
            .join("operations")
            .join(format!("{}.json", attempt.id));
        std::fs::write(path, b"{}").unwrap();
        assert!(
            attempt
                .finish(&store, &json!({"stopped":true}))
                .await
                .is_err()
        );
        let next = Attempt::prepare(
            &store,
            json!({"commit":"reviewed"}),
            &job,
            &server,
            json!({"status":"Stopped"}),
        )
        .await
        .unwrap();
        assert_ne!(attempt.id, next.id);
        assert_eq!(
            std::fs::read_dir(directory.join("operations"))
                .unwrap()
                .count(),
            3
        );
        drop(store);
        std::fs::remove_dir_all(directory).unwrap();
    }
}

pub struct Attempt {
    id: Uuid,
    hash: String,
}
impl Attempt {
    pub async fn prepare(
        store: &Store,
        evidence: Value,
        job: &Value,
        server: &Value,
        before: Value,
    ) -> Result<Self> {
        let id = Uuid::new_v4();
        let payload = json!({"schema":1,"id":id,"management":evidence,
            "created_unix":SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs(),
            "job_id":job["id"],"kind":job["kind"],"server_id":server["id"],
            "requested_effect":job["payload"],"server":server,"before":before});
        // Never overwrite an earlier attempt or take a user's path as a plan path.
        store.write("operations", id, &payload)?;
        let path = store.root.join("operations").join(format!("{id}.json"));
        let hash = file_hash(&path).await?;
        ensure!(
            store.read::<Value>("operations", id)? == Some(payload),
            "Saved runtime plan changed"
        );
        store.write(
            "jobs",
            job["id"].as_str().context("Missing job ID")?.parse()?,
            &json!({"phase":"applying","attempt_id":id,"plan_sha256":hash}),
        )?;
        Ok(Self { id, hash })
    }
    pub async fn finish(&self, store: &Store, result: &Value) -> Result<()> {
        let path = store
            .root
            .join("operations")
            .join(format!("{}.json", self.id));
        ensure!(
            file_hash(&path).await? == self.hash,
            "Runtime saved plan hash changed"
        );
        let output = store
            .root
            .join("operations")
            .join(format!("{}.receipt.json", self.id));
        crate::state::atomic(
            &output,
            &serde_json::to_vec(&json!({"plan_sha256":self.hash,"result":result}))?,
        )
    }
}
