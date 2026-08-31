use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{OpsError, Result};
use crate::manifest::{
    artifact_install_path, sha256_bytes, sha256_file, ArtifactKind, VerifiedRelease,
};
use crate::secure_fs::{
    atomic_write, copy_regular, create_directory, effective_gid, effective_uid,
    require_absolute_safe, require_directory, require_regular, sync_directory,
};

const MAX_ARTIFACT_BYTES: u64 = 1024 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallScope {
    System { service_uid: u32, service_gid: u32 },
    User,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallFault {
    None,
    AfterStage,
    AfterPublish,
    AfterCommit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallResult {
    NoOp,
    Updated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InstallIdentity {
    schema_version: u32,
    commit: String,
    manifest_sha256: String,
}

pub fn install(
    release: &VerifiedRelease,
    root: &Path,
    scope: InstallScope,
    fault: InstallFault,
) -> Result<InstallResult> {
    require_absolute_safe(root, "install root")?;
    let (uid, gid, directory_mode) = match scope {
        InstallScope::System {
            service_uid: _,
            service_gid,
        } => {
            if effective_uid() != 0 {
                return Err(OpsError::message("system install requires root"));
            }
            (0, service_gid, 0o750)
        }
        InstallScope::User => {
            if effective_uid() == 0 {
                return Err(OpsError::message("user install refuses root"));
            }
            (effective_uid(), effective_gid(), 0o700)
        }
    };
    if valid_installed_tree(root, release, uid, gid, directory_mode)? {
        return Ok(InstallResult::NoOp);
    }
    let parent = root
        .parent()
        .ok_or_else(|| OpsError::message("install root has no parent"))?;
    require_directory(parent, "install parent", None, None, None)?;
    if let Ok(metadata) = fs::symlink_metadata(root) {
        if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
            return Err(OpsError::message("existing install root is ambiguous"));
        }
    }
    let stage = sibling(root, ".lkjmc-stage")?;
    let rollback = sibling(root, ".lkjmc-rollback")?;
    create_directory(&stage, directory_mode, uid, gid)?;
    let mut prior = false;
    let mut published = false;
    let mut committed = false;
    let operation = (|| {
        stage_tree(&stage, release, uid, gid, directory_mode)?;
        if fault == InstallFault::AfterStage {
            return Err(OpsError::message("injected failure after artifact staging"));
        }
        if root.exists() {
            fs::rename(root, &rollback)
                .map_err(|error| OpsError::context("cannot retain prior install", error))?;
            prior = true;
            sync_directory(parent)?;
        }
        fs::rename(&stage, root)
            .map_err(|error| OpsError::context("cannot atomically publish install", error))?;
        published = true;
        sync_directory(parent)?;
        if fault == InstallFault::AfterPublish {
            return Err(OpsError::message(
                "injected failure after artifact publication",
            ));
        }
        if !valid_installed_tree(root, release, uid, gid, directory_mode)? {
            return Err(OpsError::message(
                "post-publication install verification failed",
            ));
        }
        committed = true;
        if fault == InstallFault::AfterCommit {
            return Err(OpsError::message(
                "injected failure after committed artifact publication",
            ));
        }
        if prior {
            remove_owned_sibling(&rollback, root, ".lkjmc-rollback")?;
            prior = false;
        }
        sync_directory(parent)?;
        Ok(InstallResult::Updated)
    })();

    if operation.is_err() && !committed {
        if published && root.exists() {
            remove_published_root(root)?;
        }
        if prior && rollback.exists() {
            fs::rename(&rollback, root)
                .map_err(|error| OpsError::context("cannot restore prior install", error))?;
            prior = false;
        }
        sync_directory(parent)?;
    }
    if stage.exists() {
        remove_owned_sibling(&stage, root, ".lkjmc-stage")?;
    }
    if rollback.exists() && !prior {
        remove_owned_sibling(&rollback, root, ".lkjmc-rollback")?;
    }
    operation
}

pub fn valid_installed_tree(
    root: &Path,
    release: &VerifiedRelease,
    uid: u32,
    gid: u32,
    directory_mode: u32,
) -> Result<bool> {
    let metadata = match fs::symlink_metadata(root) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(OpsError::context("cannot inspect install root", error)),
    };
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Ok(false);
    }
    let expected = expected_files(release)?;
    let mut actual = BTreeSet::new();
    let mut directories = BTreeSet::new();
    directories.insert(root.to_path_buf());
    collect_tree(root, root, &mut actual, &mut directories)?;
    if actual != expected.keys().cloned().collect() {
        return Ok(false);
    }
    for directory in directories {
        let value = fs::symlink_metadata(&directory)
            .map_err(|error| OpsError::context("cannot inspect installed directory", error))?;
        if !value.file_type().is_dir()
            || value.file_type().is_symlink()
            || value.uid() != uid
            || value.gid() != gid
            || value.permissions().mode() & 0o7777 != directory_mode
        {
            return Ok(false);
        }
    }
    for (relative, expected_file) in expected {
        let path = root.join(&relative);
        let value = match require_regular(
            &path,
            "installed artifact",
            Some(uid),
            Some(gid),
            Some(expected_file.mode),
            MAX_ARTIFACT_BYTES,
        ) {
            Ok(value) => value,
            Err(_) => return Ok(false),
        };
        if value.len() != expected_file.size || sha256_file(&path)? != expected_file.sha256 {
            return Ok(false);
        }
    }
    Ok(true)
}

#[derive(Debug)]
struct ExpectedFile {
    sha256: String,
    size: u64,
    mode: u32,
}

fn expected_files(release: &VerifiedRelease) -> Result<BTreeMap<PathBuf, ExpectedFile>> {
    let mut files = BTreeMap::new();
    for artifact in release.artifacts() {
        files.insert(
            artifact_install_path(artifact.kind, &artifact.path),
            ExpectedFile {
                sha256: artifact.sha256.clone(),
                size: artifact.size,
                mode: if artifact.kind == ArtifactKind::Binary {
                    0o750
                } else {
                    0o640
                },
            },
        );
    }
    for name in ["artifact-manifest.json", "artifact-manifest.json.sha256"] {
        let path = release.root.join(name);
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| OpsError::context("cannot inspect release metadata", error))?;
        files.insert(
            Path::new("meta").join(name),
            ExpectedFile {
                sha256: sha256_file(&path)?,
                size: metadata.len(),
                mode: 0o640,
            },
        );
    }
    let identity = install_identity(release)?;
    files.insert(
        PathBuf::from(".lkjmc-install.json"),
        ExpectedFile {
            sha256: sha256_bytes(&identity),
            size: identity.len() as u64,
            mode: 0o640,
        },
    );
    Ok(files)
}

fn stage_tree(
    stage: &Path,
    release: &VerifiedRelease,
    uid: u32,
    gid: u32,
    directory_mode: u32,
) -> Result<()> {
    for name in ["bin", "jars", "share", "meta"] {
        create_directory(&stage.join(name), directory_mode, uid, gid)?;
    }
    for artifact in release.artifacts() {
        let relative = artifact_install_path(artifact.kind, &artifact.path);
        let mode = if artifact.kind == ArtifactKind::Binary {
            0o750
        } else {
            0o640
        };
        copy_regular(
            &release.source.join(&artifact.path),
            &stage.join(relative),
            mode,
            uid,
            gid,
            MAX_ARTIFACT_BYTES,
        )?;
    }
    for name in ["artifact-manifest.json", "artifact-manifest.json.sha256"] {
        copy_regular(
            &release.root.join(name),
            &stage.join("meta").join(name),
            0o640,
            uid,
            gid,
            16 * 1024 * 1024,
        )?;
    }
    atomic_write(
        &stage.join(".lkjmc-install.json"),
        &install_identity(release)?,
        0o640,
        uid,
        gid,
    )?;
    for name in ["bin", "jars", "share", "meta"] {
        sync_directory(&stage.join(name))?;
    }
    sync_directory(stage)
}

fn install_identity(release: &VerifiedRelease) -> Result<Vec<u8>> {
    let mut raw = serde_json::to_vec(&InstallIdentity {
        schema_version: 1,
        commit: release.manifest.commit.clone(),
        manifest_sha256: release.manifest_sha256.clone(),
    })?;
    raw.push(b'\n');
    Ok(raw)
}

fn collect_tree(
    root: &Path,
    directory: &Path,
    files: &mut BTreeSet<PathBuf>,
    directories: &mut BTreeSet<PathBuf>,
) -> Result<()> {
    for entry in fs::read_dir(directory)
        .map_err(|error| OpsError::context("cannot enumerate installed tree", error))?
    {
        let entry =
            entry.map_err(|error| OpsError::context("cannot read installed entry", error))?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| OpsError::context("cannot inspect installed entry", error))?;
        if metadata.file_type().is_symlink() {
            return Err(OpsError::message("installed tree contains a symlink"));
        }
        if metadata.file_type().is_dir() {
            directories.insert(path.clone());
            collect_tree(root, &path, files, directories)?;
        } else if metadata.file_type().is_file() {
            files.insert(
                path.strip_prefix(root)
                    .map_err(|_| OpsError::message("installed path escapes root"))?
                    .to_path_buf(),
            );
        } else {
            return Err(OpsError::message(
                "installed tree contains a special filesystem object",
            ));
        }
    }
    Ok(())
}

fn sibling(root: &Path, prefix: &str) -> Result<PathBuf> {
    let parent = root
        .parent()
        .ok_or_else(|| OpsError::message("install root has no parent"))?;
    let name = root
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| OpsError::message("install root name is not UTF-8"))?;
    Ok(parent.join(format!("{prefix}-{name}-{}", Uuid::new_v4())))
}

fn remove_owned_sibling(path: &Path, root: &Path, prefix: &str) -> Result<()> {
    let parent = root
        .parent()
        .ok_or_else(|| OpsError::message("install root has no parent"))?;
    if path.parent() != Some(parent)
        || !path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.starts_with(prefix))
    {
        return Err(OpsError::message(
            "refusing to remove unowned install state",
        ));
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| OpsError::context("cannot inspect owned install state", error))?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(OpsError::message("owned install state is ambiguous"));
    }
    fs::remove_dir_all(path)
        .map_err(|error| OpsError::context("cannot remove owned install state", error))
}

fn remove_published_root(root: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(root)
        .map_err(|error| OpsError::context("cannot inspect failed publication", error))?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(OpsError::message("failed publication root is ambiguous"));
    }
    fs::remove_dir_all(root)
        .map_err(|error| OpsError::context("cannot remove failed publication", error))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use serde_json::json;

    use super::*;
    use crate::manifest::{sha256_bytes, OPERATIONS_RELEASE_INVENTORY};

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new() -> Result<Self> {
            let path = std::env::temp_dir().join(format!("lkjmc-ops-install-{}", Uuid::new_v4()));
            fs::create_dir(&path)?;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
            Ok(Self(path))
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn publish_noop_and_interruption_preserve_exact_tree() -> Result<()> {
        if effective_uid() == 0 {
            return Ok(());
        }
        let test = TestRoot::new()?;
        let first_root = test.0.join("release-first");
        let first_digest = fixture_release(&first_root, b'1')?;
        let first = VerifiedRelease::load_anchored(&first_root, &first_digest)?;
        let installed = test.0.join("installed");
        assert_eq!(
            install(&first, &installed, InstallScope::User, InstallFault::None)?,
            InstallResult::Updated
        );
        let before = fs::metadata(installed.join("bin/lkjmc-ops"))?;
        assert_eq!(
            install(&first, &installed, InstallScope::User, InstallFault::None)?,
            InstallResult::NoOp
        );
        let after = fs::metadata(installed.join("bin/lkjmc-ops"))?;
        assert_eq!(
            (before.ino(), before.mtime_nsec()),
            (after.ino(), after.mtime_nsec())
        );

        let changed_root = test.0.join("release-changed");
        let changed_digest = fixture_release(&changed_root, b'2')?;
        let changed = VerifiedRelease::load_anchored(&changed_root, &changed_digest)?;
        let error = install(
            &changed,
            &installed,
            InstallScope::User,
            InstallFault::AfterPublish,
        )
        .err()
        .ok_or_else(|| OpsError::message("interrupted install unexpectedly passed"))?;
        assert!(error.to_string().contains("after artifact publication"));
        assert_eq!(fs::read(installed.join("bin/lkjmc-ops"))?, b"1-lkjmc-ops\n");
        assert!(!fs::read_dir(&test.0)?.any(|entry| {
            entry
                .ok()
                .and_then(|value| value.file_name().into_string().ok())
                .is_some_and(|name| name.starts_with(".lkjmc-"))
        }));
        Ok(())
    }

    fn fixture_release(root: &Path, marker: u8) -> Result<String> {
        fs::create_dir(root)?;
        fs::set_permissions(root, fs::Permissions::from_mode(0o700))?;
        let source = root.join("source");
        fs::create_dir(&source)?;
        fs::set_permissions(&source, fs::Permissions::from_mode(0o700))?;
        let mut artifacts = Vec::new();
        for (name, kind) in OPERATIONS_RELEASE_INVENTORY {
            let bytes = [
                vec![marker],
                b"-".to_vec(),
                name.as_bytes().to_vec(),
                b"\n".to_vec(),
            ]
            .concat();
            let path = source.join(name);
            fs::write(&path, &bytes)?;
            let mode = if *kind == ArtifactKind::Binary {
                0o700
            } else {
                0o600
            };
            fs::set_permissions(&path, fs::Permissions::from_mode(mode))?;
            artifacts.push(json!({
                "component": name,
                "kind": kind,
                "path": name,
                "provenance": format!("pinned build at {}", "a".repeat(40)),
                "sha256": sha256_bytes(&bytes),
                "size": bytes.len(),
                "source": format!("fixture/{name}")
            }));
        }
        let mut raw = serde_json::to_vec_pretty(&json!({
            "schemaVersion": 1,
            "commit": if marker == b'1' { "a".repeat(40) } else { "b".repeat(40) },
            "artifacts": artifacts,
            "components": [], "contracts": [], "images": []
        }))?;
        raw.push(b'\n');
        let digest = sha256_bytes(&raw);
        fs::write(root.join("artifact-manifest.json"), &raw)?;
        fs::write(
            root.join("artifact-manifest.json.sha256"),
            format!("{digest}  artifact-manifest.json\n"),
        )?;
        Ok(digest)
    }
}
