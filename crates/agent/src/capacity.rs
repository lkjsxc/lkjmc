//! Count live VM reservations as well as queued desired state. Never infer
//! available host resources from the application database alone.
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::CString,
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
    path::Path,
};
use uuid::Uuid;

pub const MIB: u64 = 1024 * 1024;
const METADATA_RESERVE: u64 = 64 * MIB;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Resources {
    pub memory: u128,
    pub cpu: u128,
    pub storage: u128,
}
impl Resources {
    pub fn requested(server: &Value) -> Result<Self> {
        let positive = |key: &str| -> Result<u128> {
            let n = server[key].as_u64().context("Missing resource limit")?;
            ensure!(n > 0, "Resource limit must be positive");
            Ok(u128::from(n))
        };
        Ok(Self {
            memory: positive("memory_mib")? * u128::from(MIB),
            cpu: positive("cpu_millis")?.div_ceil(1000),
            storage: positive("storage_mib")? * u128::from(MIB),
        })
    }
    pub fn instance(instance: &Value) -> Result<Self> {
        let config = &instance["expanded_config"];
        let cpu = config["limits.cpu"]
            .as_str()
            .context("VM CPU limit is absent")?
            .parse::<u64>()
            .context("Expected a bounded VM CPU count")?;
        ensure!(cpu > 0, "VM CPU limit must be positive");
        Ok(Self {
            memory: size(
                config["limits.memory"]
                    .as_str()
                    .context("VM RAM limit is absent")?,
            )?,
            cpu: cpu.into(),
            storage: size(
                instance["expanded_devices"]["root"]["size"]
                    .as_str()
                    .context("VM disk limit is absent")?,
            )?,
        })
    }
    fn maximum(self, other: Self) -> Self {
        Self {
            memory: self.memory.max(other.memory),
            cpu: self.cpu.max(other.cpu),
            storage: self.storage.max(other.storage),
        }
    }
}

fn size(value: &str) -> Result<u128> {
    for (unit, factor) in [
        ("TiB", 1024_u128.pow(4)),
        ("GiB", 1024_u128.pow(3)),
        ("MiB", 1024_u128.pow(2)),
        ("KiB", 1024_u128),
        ("TB", 1000_u128.pow(4)),
        ("GB", 1000_u128.pow(3)),
        ("MB", 1000_u128.pow(2)),
        ("kB", 1000_u128),
        ("B", 1),
    ] {
        if let Some(n) = value.strip_suffix(unit) {
            let n = n.parse::<u64>().context("Invalid VM resource size")?;
            ensure!(n > 0, "VM resource limit must be positive");
            return Ok(u128::from(n) * factor);
        }
    }
    let n = value
        .parse::<u64>()
        .context("VM resource size needs an explicit bound")?;
    ensure!(n > 0, "VM resource limit must be positive");
    Ok(n.into())
}

/// A live VM consumes its limits even when Core thinks it is stopped. Pending
/// starts and incomplete creates also reserve resources, with one entry per ID.
pub fn tenant_usage(
    servers: &[Value],
    instances: &[Value],
    bindings: &BTreeSet<Uuid>,
    target: &Value,
) -> Result<Resources> {
    let id = |v: &Value| -> Result<Uuid> { Ok(v.as_str().context("Missing server ID")?.parse()?) };
    let target_id = id(&target["id"])?;
    let mut registry = BTreeMap::new();
    for server in servers.iter().filter(|s| s["kind"] == "custom") {
        ensure!(
            registry.insert(id(&server["id"])?, server).is_none(),
            "Duplicate server ID"
        );
    }
    registry.insert(target_id, target);
    let mut actual = BTreeMap::new();
    for instance in instances {
        let key = id(&instance["expanded_config"]["user.lkjmc.server-id"])?;
        ensure!(
            actual.insert(key, instance).is_none(),
            "Duplicate VM ownership"
        );
    }
    let keys: BTreeSet<Uuid> = registry
        .keys()
        .chain(actual.keys())
        .chain(bindings.iter())
        .copied()
        .collect();
    let mut usage = Resources::default();
    for key in keys {
        let recorded = registry.get(&key);
        let live = actual.get(&key);
        ensure!(
            recorded.is_some() || live.is_some(),
            "Binding has neither a registered nor a live VM"
        );
        let desired = recorded
            .map(|s| Resources::requested(s))
            .transpose()?
            .unwrap_or_default();
        let observed = live
            .map(|i| Resources::instance(i))
            .transpose()?
            .unwrap_or_default();
        let reservation = desired.maximum(observed);
        if live.is_some() || bindings.contains(&key) || key == target_id {
            usage.storage = usage
                .storage
                .checked_add(reservation.storage)
                .context("Storage reservation overflow")?;
        }
        if key == target_id
            || recorded.is_some_and(|s| s["desired"] == "running" || !s["inspection"].is_null())
            || live.is_some_and(|i| i["status"] != "Stopped")
        {
            usage.memory = usage
                .memory
                .checked_add(reservation.memory)
                .context("Memory reservation overflow")?;
            usage.cpu = usage
                .cpu
                .checked_add(reservation.cpu)
                .context("CPU reservation overflow")?;
        }
    }
    Ok(usage)
}

pub fn available_memory() -> Result<u128> {
    let memory = std::fs::read_to_string("/proc/meminfo")?;
    let kb = memory
        .lines()
        .find_map(|line| {
            line.strip_prefix("MemAvailable:")
                .and_then(|v| v.split_whitespace().next())
                .and_then(|v| v.parse::<u64>().ok())
        })
        .context("Cannot measure available host RAM")?;
    Ok(u128::from(kb) * 1024)
}

pub fn free_bytes(path: &Path) -> Result<u128> {
    let path = CString::new(path.as_os_str().as_bytes())?;
    let mut result = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    // Both pointers remain valid for the duration of statvfs. Read only after success.
    let status = unsafe { libc::statvfs(path.as_ptr(), result.as_mut_ptr()) };
    if status != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let result = unsafe { result.assume_init() };
    Ok(u128::from(result.f_bavail) * u128::from(result.f_frsize))
}

pub fn preserve_pool(path: &Path, reserve_mib: u64, additional: u128) -> Result<()> {
    ensure!(
        free_bytes(path)? >= u128::from(reserve_mib) * u128::from(MIB) + additional,
        "実測した保存領域の空きが不足しています。既存データ用の余裕を残すため、この操作を待機します。"
    );
    Ok(())
}

/// The Incus custom-volume API does not report an enforced filesystem quota on
/// this Btrfs pool. Bound all agent files explicitly, including partial downloads.
pub fn archive_available(root: &Path, max_mib: u64) -> Result<u64> {
    let mut directories = vec![root.to_owned()];
    let mut used = 0_u128;
    let device = std::fs::symlink_metadata(root)?.dev();
    let mut entries = 0_u64;
    while let Some(directory) = directories.pop() {
        let info = std::fs::symlink_metadata(&directory)?;
        ensure!(
            info.is_dir() && info.dev() == device,
            "Archive storage contains an indirection"
        );
        for entry in std::fs::read_dir(directory)? {
            entries += 1;
            ensure!(
                entries <= 1_000_000,
                "Archive inventory is too large to measure safely"
            );
            let path = entry?.path();
            let info = std::fs::symlink_metadata(&path)?;
            ensure!(
                info.dev() == device,
                "Archive storage crossed a mount boundary"
            );
            if info.is_dir() {
                directories.push(path);
            } else {
                ensure!(
                    info.is_file() && info.nlink() == 1,
                    "Archive storage contains a link or special file"
                );
                used += u128::from(info.len().max(info.blocks().saturating_mul(512)));
            }
        }
    }
    let budget = u128::from(max_mib) * u128::from(MIB);
    ensure!(
        used + u128::from(METADATA_RESERVE) < budget,
        "バックアップ保存領域が上限に達しています。"
    );
    Ok((budget - used - u128::from(METADATA_RESERVE)).min(u128::from(u64::MAX)) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn server(id: Uuid, desired: &str) -> Value {
        json!({"id":id,"kind":"custom","desired":desired,"memory_mib":2048,"cpu_millis":1500,"storage_mib":8192})
    }
    fn instance(id: Uuid, status: &str) -> Value {
        json!({"status":status,"expanded_config":{"user.lkjmc.server-id":id,"limits.memory":"3GiB","limits.cpu":"3"},
            "expanded_devices":{"root":{"size":"10GiB"}}})
    }
    #[test]
    fn counts_live_orphans_pending_starts_and_temporary_boots_once() {
        let (target, live, queued, orphan) = (
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        let servers = vec![
            server(target, "stopped"),
            server(live, "stopped"),
            server(queued, "running"),
        ];
        let instances = vec![
            instance(target, "Stopped"),
            instance(live, "Running"),
            instance(orphan, "Frozen"),
        ];
        let usage = tenant_usage(
            &servers,
            &instances,
            &BTreeSet::from([target, live]),
            &servers[0],
        )
        .unwrap();
        assert_eq!(
            usage,
            Resources {
                memory: 11 * 1024 * u128::from(MIB),
                cpu: 11,
                storage: 30 * 1024 * u128::from(MIB)
            }
        );
        assert!(
            tenant_usage(
                &servers,
                &instances,
                &BTreeSet::from([Uuid::new_v4()]),
                &servers[0]
            )
            .is_err()
        );
    }
    #[test]
    fn malformed_or_unbounded_resources_do_not_turn_into_free_capacity() {
        let id = Uuid::new_v4();
        let mut value = instance(id, "Running");
        for field in ["limits.memory", "limits.cpu"] {
            value["expanded_config"][field] = json!("");
            assert!(Resources::instance(&value).is_err());
        }
        let mut huge = server(id, "running");
        huge["memory_mib"] = json!(u64::MAX);
        assert!(Resources::requested(&huge).unwrap().memory > u128::from(u64::MAX));
        huge["cpu_millis"] = json!(0);
        assert!(Resources::requested(&huge).is_err());
    }
    #[test]
    fn measures_partial_archives_and_refuses_links_without_following_them() {
        let root = std::env::temp_dir().join(format!("lkjmc-capacity-{}", Uuid::new_v4()));
        std::fs::create_dir(&root).unwrap();
        let before = archive_available(&root, 128).unwrap();
        let partial = root.join("backup.partial");
        std::fs::write(&partial, vec![0; MIB as usize]).unwrap();
        assert!(archive_available(&root, 128).unwrap() <= before - MIB);
        std::os::unix::fs::symlink("/", root.join("escape")).unwrap();
        assert!(archive_available(&root, 128).is_err());
        std::fs::remove_file(root.join("escape")).unwrap();
        std::fs::hard_link(&partial, root.join("duplicate")).unwrap();
        assert!(archive_available(&root, 128).is_err());
        assert!(free_bytes(&root).unwrap() > 0);
        std::fs::remove_dir_all(root).unwrap();
    }
}
