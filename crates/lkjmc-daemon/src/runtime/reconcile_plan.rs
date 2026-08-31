use std::fs::OpenOptions;
use std::io::Read;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use lkjmc_core::instance::InstanceKind;
use lkjmc_core::runtime_lifecycle::{LifecycleDecision, RuntimeIntent, RuntimeObserved};

use crate::app::AppState;
use crate::runtime::instance_launch::LaunchSpec;
use crate::runtime::reconcile::{RuntimeGoal, EFFECT_DEADLINE};
use crate::runtime::RuntimeObservation;

pub(super) struct PreparedStart {
    launch: LaunchSpec,
    work_dir: PathBuf,
}

pub(super) fn desired_intent(goal: RuntimeGoal) -> Result<RuntimeIntent, String> {
    match goal {
        RuntimeGoal::Running => Ok(RuntimeIntent::Running),
        RuntimeGoal::Stopped => Ok(RuntimeIntent::Stopped),
        RuntimeGoal::Deleted => Ok(RuntimeIntent::Deleted),
        RuntimeGoal::Observe => Err("observe has no desired intent".to_string()),
    }
}

pub(super) fn prepare_start(state: &AppState, id: &str) -> Result<PreparedStart, String> {
    let (kind, config, launch) = {
        let mut client = state.database_connection()?;
        let instance = lkjmc_store::instance::get(&mut client, id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("instance not found: {id}"))?;
        let config = lkjmc_store::instance::config(&mut client, id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("instance config not found: {id}"))?;
        let launch =
            crate::runtime::instance_launch::launch(state, &mut client, &instance.kind, &config)?;
        (instance.kind, config, launch)
    };
    let planned_work_dir = Path::new(&state.data_root()).join(id);
    verify_start_eula(&kind, &planned_work_dir)?;
    let work_dir = crate::templates::render_instance(state, id, &kind, &config)?;
    verify_start_eula(&kind, &work_dir)?;
    Ok(PreparedStart { launch, work_dir })
}

fn verify_start_eula(kind: &str, work_dir: &Path) -> Result<(), String> {
    let kind = InstanceKind::from_wire(kind)
        .ok_or_else(|| format!("unsupported instance kind before start: {kind}"))?;
    if !kind.requires_minecraft_eula() {
        return Ok(());
    }
    let directory = std::fs::symlink_metadata(work_dir)
        .map_err(|error| format!("inspect managed instance directory: {error}"))?;
    if !directory.file_type().is_dir() || directory.file_type().is_symlink() {
        return Err("managed instance directory is not a non-symlink directory".to_string());
    }
    let path = work_dir.join("eula.txt");
    let before = std::fs::symlink_metadata(&path)
        .map_err(|error| format!("Minecraft EULA is not materialized for {kind:?}: {error}"))?;
    let expected_uid = expected_eula_uid(&directory);
    if !before.file_type().is_file()
        || before.file_type().is_symlink()
        || before.uid() != expected_uid
        || before.gid() != directory.gid()
        || before.permissions().mode() & 0o777 != 0o640
        || before.len() != b"eula=true\n".len() as u64
    {
        return Err(
            "materialized Minecraft EULA identity, ownership, mode, or size differs".to_string(),
        );
    }
    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW);
    let mut file = options
        .open(&path)
        .map_err(|error| format!("open materialized Minecraft EULA: {error}"))?;
    let opened = file
        .metadata()
        .map_err(|error| format!("reinspect materialized Minecraft EULA: {error}"))?;
    if (before.dev(), before.ino(), before.len()) != (opened.dev(), opened.ino(), opened.len()) {
        return Err("materialized Minecraft EULA identity changed during verification".to_string());
    }
    let mut bytes = Vec::with_capacity(opened.len() as usize);
    file.read_to_end(&mut bytes)
        .map_err(|error| format!("read materialized Minecraft EULA: {error}"))?;
    if bytes != b"eula=true\n" {
        return Err("materialized Minecraft EULA content differs".to_string());
    }
    Ok(())
}

#[cfg(not(test))]
fn expected_eula_uid(_directory: &std::fs::Metadata) -> u32 {
    0
}

#[cfg(test)]
fn expected_eula_uid(directory: &std::fs::Metadata) -> u32 {
    directory.uid()
}

pub(super) fn perform(
    state: &AppState,
    id: &str,
    decision: LifecycleDecision,
    prepared: Option<&PreparedStart>,
    stop_config: Option<&serde_json::Value>,
) -> Result<RuntimeObservation, String> {
    let runtime = state.runtime();
    match decision {
        LifecycleDecision::Start => {
            let prepared = prepared.ok_or("start plan missing")?;
            let observation = runtime.runtime_start(
                id,
                &prepared.launch.command,
                &prepared.launch.args,
                &prepared.launch.env,
                &state.log_root(),
                &prepared.work_dir,
                EFFECT_DEADLINE,
            )?;
            if observation.healthy {
                Ok(observation)
            } else {
                let detail = observation
                    .message
                    .unwrap_or_else(|| "process did not become healthy after start".to_string());
                Err(format!("instance {id} failed to start: {detail}"))
            }
        }
        LifecycleDecision::Stop => {
            if let Some(config) = stop_config {
                let _ = crate::runtime::rcon::stop_from_config(config);
            }
            runtime.runtime_stop(id, EFFECT_DEADLINE)
        }
        LifecycleDecision::Delete => runtime.runtime_delete(id, EFFECT_DEADLINE),
        other => Err(format!("runtime decision cannot perform effect: {other:?}")),
    }
}

pub(super) fn observed_kind(observation: Option<&RuntimeObservation>) -> RuntimeObserved {
    match observation {
        None => RuntimeObserved::Absent,
        Some(value) if value.healthy => RuntimeObserved::Running,
        Some(value) if value.observed_state.contains("absent") => RuntimeObserved::Absent,
        Some(_) => RuntimeObserved::Unhealthy,
    }
}

pub(super) fn intent_name(intent: RuntimeIntent) -> &'static str {
    match intent {
        RuntimeIntent::Running => "running",
        RuntimeIntent::Stopped => "stopped",
        RuntimeIntent::Deleted => "deleted",
    }
}

pub(super) fn observed_name(observation: Option<&RuntimeObservation>) -> &'static str {
    match observed_kind(observation) {
        RuntimeObserved::Running => "running",
        RuntimeObserved::Absent => "absent",
        RuntimeObserved::Unhealthy => "unhealthy",
        RuntimeObserved::Unknown => "unknown",
    }
}

pub(super) fn decision_name(decision: LifecycleDecision) -> &'static str {
    match decision {
        LifecycleDecision::Start => "start",
        LifecycleDecision::Stop => "stop",
        LifecycleDecision::Delete => "delete",
        LifecycleDecision::ObservePending => "observe",
        LifecycleDecision::Noop => "observe",
        LifecycleDecision::Unsupported => "observe",
    }
}

#[cfg(test)]
mod eula_tests {
    use super::verify_start_eula;
    use std::fs;
    use std::os::unix::fs::{symlink, PermissionsExt};

    #[test]
    fn eula_is_required_only_for_minecraft_server_kinds() -> Result<(), String> {
        let root = test_root()?;
        verify_start_eula("velocity", &root)?;
        let missing = verify_start_eula("paper", &root)
            .err()
            .ok_or("missing EULA unexpectedly passed")?;
        assert!(missing.contains("not materialized"));
        write_eula(&root, 0o640, b"eula=true\n")?;
        verify_start_eula("paper", &root)?;
        fs::remove_dir_all(root).map_err(|error| error.to_string())
    }

    #[test]
    fn eula_rejects_wrong_mode_content_and_symlink() -> Result<(), String> {
        let root = test_root()?;
        write_eula(&root, 0o600, b"eula=true\n")?;
        let wrong_mode = verify_start_eula("folia", &root)
            .err()
            .ok_or("wrong EULA mode unexpectedly passed")?;
        assert!(wrong_mode.contains("ownership, mode, or size"));
        write_eula(&root, 0o640, b"eula=oops\n")?;
        let wrong_content = verify_start_eula("folia", &root)
            .err()
            .ok_or("wrong EULA content unexpectedly passed")?;
        assert!(wrong_content.contains("content differs"));
        fs::remove_file(root.join("eula.txt")).map_err(|error| error.to_string())?;
        let unrelated = root.join("unrelated");
        fs::write(&unrelated, b"eula=true\n").map_err(|error| error.to_string())?;
        symlink(&unrelated, root.join("eula.txt")).map_err(|error| error.to_string())?;
        let symlinked = verify_start_eula("folia", &root)
            .err()
            .ok_or("symlink EULA unexpectedly passed")?;
        assert!(symlinked.contains("identity, ownership, mode, or size"));
        fs::remove_dir_all(root).map_err(|error| error.to_string())
    }

    fn test_root() -> Result<std::path::PathBuf, String> {
        let root = std::env::temp_dir().join(format!(
            "lkjmc-start-eula-{}",
            uuid::Uuid::new_v4().simple()
        ));
        fs::create_dir(&root).map_err(|error| error.to_string())?;
        fs::set_permissions(&root, fs::Permissions::from_mode(0o750))
            .map_err(|error| error.to_string())?;
        Ok(root)
    }

    fn write_eula(root: &std::path::Path, mode: u32, bytes: &[u8]) -> Result<(), String> {
        let path = root.join("eula.txt");
        fs::write(&path, bytes).map_err(|error| error.to_string())?;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
            .map_err(|error| error.to_string())
    }
}
