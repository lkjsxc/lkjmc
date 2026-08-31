use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::error::{OpsError, Result};

const MAX_MANIFEST_BYTES: u64 = 16 * 1024 * 1024;
const MAX_DIGEST_BYTES: u64 = 64 * 1024 * 1024 * 1024;

pub const OPERATIONS_RELEASE_INVENTORY: &[(&str, ArtifactKind)] = &[
    ("lkjmc", ArtifactKind::Binary),
    ("lkjmc-common.jar", ArtifactKind::Jar),
    ("lkjmc-daemon", ArtifactKind::Binary),
    ("lkjmc-daemon.service", ArtifactKind::Config),
    ("lkjmc-deployment-fence.conf", ArtifactKind::Config),
    ("lkjmc-ops", ArtifactKind::Binary),
    ("lkjmc-paper.jar", ArtifactKind::Jar),
    ("lkjmc-velocity.jar", ArtifactKind::Jar),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactKind {
    Binary,
    Jar,
    Config,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseArtifact {
    pub component: String,
    pub kind: ArtifactKind,
    pub path: String,
    pub provenance: String,
    pub sha256: String,
    pub size: u64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReleaseManifest {
    pub schema_version: u32,
    pub commit: String,
    pub artifacts: Vec<ReleaseArtifact>,
    #[serde(default)]
    pub components: Vec<Value>,
    #[serde(default)]
    pub contracts: Vec<Value>,
    #[serde(default)]
    pub images: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct VerifiedRelease {
    pub root: PathBuf,
    pub source: PathBuf,
    pub manifest_path: PathBuf,
    pub manifest_sha256: String,
    pub manifest: ReleaseManifest,
    artifacts: BTreeMap<String, ReleaseArtifact>,
}

impl VerifiedRelease {
    pub fn load_anchored(root: &Path, expected_manifest_sha256: &str) -> Result<Self> {
        if !root.is_absolute() {
            return Err(OpsError::message("release root must be absolute"));
        }
        require_directory(root, "release root")?;
        let source = root.join("source");
        require_directory(&source, "release source")?;
        let manifest_path = root.join("artifact-manifest.json");
        let sidecar_path = root.join("artifact-manifest.json.sha256");
        let (manifest, manifest_sha256) =
            read_anchored_manifest(&manifest_path, &sidecar_path, expected_manifest_sha256)?;
        let artifacts = manifest
            .artifacts
            .iter()
            .cloned()
            .map(|artifact| (artifact.path.clone(), artifact))
            .collect::<BTreeMap<_, _>>();
        verify_source_closure(&source, &artifacts)?;
        Ok(Self {
            root: root.to_path_buf(),
            source,
            manifest_path,
            manifest_sha256,
            manifest,
            artifacts,
        })
    }

    pub fn artifact(&self, name: &str) -> Result<&ReleaseArtifact> {
        self.artifacts
            .get(name)
            .ok_or_else(|| OpsError::message(format!("release artifact is missing: {name}")))
    }

    pub fn artifacts(&self) -> impl Iterator<Item = &ReleaseArtifact> {
        self.artifacts.values()
    }
}

pub fn read_anchored_manifest(
    manifest_path: &Path,
    sidecar_path: &Path,
    expected_manifest_sha256: &str,
) -> Result<(ReleaseManifest, String)> {
    let (manifest, manifest_sha256) =
        read_anchored_manifest_base(manifest_path, sidecar_path, expected_manifest_sha256)?;
    validate_operations_inventory(&manifest)?;
    Ok((manifest, manifest_sha256))
}

pub fn read_anchored_source_manifest(
    manifest_path: &Path,
    sidecar_path: &Path,
    expected_manifest_sha256: &str,
) -> Result<(ReleaseManifest, String)> {
    read_anchored_manifest_base(manifest_path, sidecar_path, expected_manifest_sha256)
}

fn read_anchored_manifest_base(
    manifest_path: &Path,
    sidecar_path: &Path,
    expected_manifest_sha256: &str,
) -> Result<(ReleaseManifest, String)> {
    require_sha256(expected_manifest_sha256, "manifest SHA-256")?;
    let raw = read_regular_bounded(manifest_path, "artifact manifest", MAX_MANIFEST_BYTES)?;
    let manifest_sha256 = sha256_bytes(&raw);
    if manifest_sha256 != expected_manifest_sha256 {
        return Err(OpsError::message(
            "artifact manifest differs from the anchored SHA-256",
        ));
    }
    let sidecar = read_regular_bounded(sidecar_path, "manifest sidecar", 256)?;
    let expected_sidecar = format!("{manifest_sha256}  artifact-manifest.json\n");
    if sidecar != expected_sidecar.as_bytes() {
        return Err(OpsError::message("artifact manifest sidecar differs"));
    }
    let manifest: ReleaseManifest = serde_json::from_slice(&raw)
        .map_err(|error| OpsError::context("invalid artifact manifest", error))?;
    validate_manifest_fields(&manifest)?;
    Ok((manifest, manifest_sha256))
}

pub fn sha256_file(path: &Path) -> Result<String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| OpsError::context("cannot inspect digest input", error))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(OpsError::message(format!(
            "digest input is not a regular file: {}",
            path.display()
        )));
    }
    if metadata.len() > MAX_DIGEST_BYTES {
        return Err(OpsError::message(format!(
            "digest input exceeds the 64-GiB verification bound: {}",
            path.display()
        )));
    }
    let mut input =
        File::open(path).map_err(|error| OpsError::context("cannot open digest input", error))?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let count = input
            .read(&mut buffer)
            .map_err(|error| OpsError::context("cannot read digest input", error))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn artifact_install_path(kind: ArtifactKind, name: &str) -> PathBuf {
    match kind {
        ArtifactKind::Binary => Path::new("bin").join(name),
        ArtifactKind::Jar => Path::new("jars").join(name),
        ArtifactKind::Config => Path::new("share").join(name),
    }
}

fn validate_manifest_fields(manifest: &ReleaseManifest) -> Result<()> {
    if manifest.schema_version != 1 {
        return Err(OpsError::message("unsupported artifact manifest schema"));
    }
    require_hex(&manifest.commit, 40, "release commit")?;
    if manifest.artifacts.is_empty() || manifest.artifacts.len() > 128 {
        return Err(OpsError::message(
            "release artifact count must be between 1 and 128",
        ));
    }
    let mut observed = BTreeMap::new();
    for artifact in &manifest.artifacts {
        require_safe_name(&artifact.path)?;
        require_sha256(&artifact.sha256, "artifact SHA-256")?;
        if artifact.size > 1024 * 1024 * 1024 {
            return Err(OpsError::message(format!(
                "release artifact exceeds size bound: {}",
                artifact.path
            )));
        }
        if (artifact.kind == ArtifactKind::Jar) != artifact.path.ends_with(".jar") {
            return Err(OpsError::message(format!(
                "release artifact kind differs from path: {}",
                artifact.path
            )));
        }
        if artifact.component.trim().is_empty()
            || artifact.provenance.trim().is_empty()
            || artifact.source.trim().is_empty()
        {
            return Err(OpsError::message(format!(
                "release artifact metadata is empty: {}",
                artifact.path
            )));
        }
        if observed
            .insert(artifact.path.as_str(), artifact.kind)
            .is_some()
        {
            return Err(OpsError::message(format!(
                "duplicate release artifact: {}",
                artifact.path
            )));
        }
    }
    Ok(())
}

fn validate_operations_inventory(manifest: &ReleaseManifest) -> Result<()> {
    let observed = manifest
        .artifacts
        .iter()
        .map(|artifact| (artifact.path.as_str(), artifact.kind))
        .collect::<BTreeMap<_, _>>();
    let expected = OPERATIONS_RELEASE_INVENTORY
        .iter()
        .copied()
        .collect::<BTreeMap<_, _>>();
    if observed != expected {
        return Err(OpsError::message(
            "release artifact set differs from the Rust operations contract",
        ));
    }
    Ok(())
}

fn verify_source_closure(
    source: &Path,
    artifacts: &BTreeMap<String, ReleaseArtifact>,
) -> Result<()> {
    let mut actual = BTreeSet::new();
    for entry in fs::read_dir(source)
        .map_err(|error| OpsError::context("cannot enumerate release source", error))?
    {
        let entry = entry.map_err(|error| OpsError::context("cannot read release entry", error))?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| OpsError::message("release artifact name is not UTF-8"))?;
        require_safe_name(&name)?;
        let metadata = entry
            .path()
            .symlink_metadata()
            .map_err(|error| OpsError::context("cannot inspect release artifact", error))?;
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return Err(OpsError::message(format!(
                "release source contains a non-regular artifact: {name}"
            )));
        }
        actual.insert(name);
    }
    if actual != artifacts.keys().cloned().collect() {
        return Err(OpsError::message(
            "release source closure differs from the manifest",
        ));
    }
    for artifact in artifacts.values() {
        let path = source.join(&artifact.path);
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| OpsError::context("cannot inspect release artifact", error))?;
        let mode = metadata.permissions().mode() & 0o7777;
        let expected_mode = match artifact.kind {
            ArtifactKind::Binary => 0o700,
            ArtifactKind::Jar | ArtifactKind::Config => 0o600,
        };
        if mode != expected_mode {
            return Err(OpsError::message(format!(
                "release artifact mode differs: {}",
                artifact.path
            )));
        }
        if metadata.len() != artifact.size || sha256_file(&path)? != artifact.sha256 {
            return Err(OpsError::message(format!(
                "release artifact bytes differ: {}",
                artifact.path
            )));
        }
    }
    Ok(())
}

fn read_regular_bounded(path: &Path, label: &str, max_bytes: u64) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| OpsError::context(&format!("missing {label}"), error))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(OpsError::message(format!(
            "{label} is not a regular file: {}",
            path.display()
        )));
    }
    if metadata.len() > max_bytes {
        return Err(OpsError::message(format!("{label} exceeds its size bound")));
    }
    fs::read(path).map_err(|error| OpsError::context(&format!("cannot read {label}"), error))
}

fn require_directory(path: &Path, label: &str) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| OpsError::context(&format!("missing {label}"), error))?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(OpsError::message(format!(
            "{label} is not a non-symlink directory: {}",
            path.display()
        )));
    }
    if metadata.mode() & 0o022 != 0 {
        return Err(OpsError::message(format!(
            "{label} is group/other writable: {}",
            path.display()
        )));
    }
    Ok(())
}

fn require_safe_name(name: &str) -> Result<()> {
    let path = Path::new(name);
    if name.is_empty()
        || name == "."
        || name == ".."
        || path.components().count() != 1
        || path.file_name().and_then(|value| value.to_str()) != Some(name)
    {
        return Err(OpsError::message(format!(
            "unsafe release artifact path: {name}"
        )));
    }
    Ok(())
}

fn require_sha256(value: &str, label: &str) -> Result<()> {
    require_hex(value, 64, label)
}

fn require_hex(value: &str, length: usize, label: &str) -> Result<()> {
    if value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(OpsError::message(format!(
            "{label} must be {length} lowercase hexadecimal characters"
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new() -> Result<Self> {
            let path = std::env::temp_dir().join(format!("lkjmc-ops-manifest-{}", Uuid::new_v4()));
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

    fn release(root: &Path) -> Result<String> {
        let source = root.join("source");
        fs::create_dir(&source)?;
        fs::set_permissions(&source, fs::Permissions::from_mode(0o700))?;
        let artifacts = OPERATIONS_RELEASE_INVENTORY
            .iter()
            .map(|(name, kind)| {
                let path = source.join(name);
                let bytes = format!("exact-{name}\n").into_bytes();
                fs::write(&path, &bytes)?;
                let mode = if *kind == ArtifactKind::Binary {
                    0o700
                } else {
                    0o600
                };
                fs::set_permissions(&path, fs::Permissions::from_mode(mode))?;
                Ok(json!({
                    "component": name,
                    "kind": kind,
                    "path": name,
                    "provenance": format!("pinned build at {}", "a".repeat(40)),
                    "sha256": sha256_bytes(&bytes),
                    "size": bytes.len(),
                    "source": format!("fixture/{name}")
                }))
            })
            .collect::<Result<Vec<_>>>()?;
        let raw = serde_json::to_vec_pretty(&json!({
            "schemaVersion": 1,
            "commit": "a".repeat(40),
            "artifacts": artifacts,
            "components": [],
            "contracts": [],
            "images": []
        }))?;
        let mut raw = raw;
        raw.push(b'\n');
        let digest = sha256_bytes(&raw);
        fs::write(root.join("artifact-manifest.json"), &raw)?;
        fs::write(
            root.join("artifact-manifest.json.sha256"),
            format!("{digest}  artifact-manifest.json\n"),
        )?;
        Ok(digest)
    }

    #[test]
    fn anchored_release_verifies_exact_new_inventory() -> Result<()> {
        let root = TestRoot::new()?;
        let digest = release(&root.0)?;
        let verified = VerifiedRelease::load_anchored(&root.0, &digest)?;
        assert_eq!(verified.manifest.commit, "a".repeat(40));
        assert_eq!(
            verified.artifacts().count(),
            OPERATIONS_RELEASE_INVENTORY.len()
        );
        assert!(verified.artifact("lkjmc-ops").is_ok());
        Ok(())
    }

    #[test]
    fn anchored_release_rejects_changed_bytes() -> Result<()> {
        let root = TestRoot::new()?;
        let digest = release(&root.0)?;
        fs::write(root.0.join("source/lkjmc-ops"), b"changed\n")?;
        let error = VerifiedRelease::load_anchored(&root.0, &digest)
            .err()
            .ok_or_else(|| OpsError::message("changed release unexpectedly passed"))?;
        assert!(error.to_string().contains("bytes differ"));
        Ok(())
    }
}
