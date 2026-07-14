use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::app::AppState;

type ArchiveMembers = (
    Vec<String>,
    BTreeMap<String, Vec<u8>>,
    BTreeMap<String, u32>,
);

#[test]
fn support_bundle_pass_is_private_sorted_hashed_and_redacted() -> Result<(), String> {
    let Some(database) = database()? else {
        return Ok(());
    };
    let root = unique_root();
    let logs = root.join("logs");
    fs::create_dir_all(&logs).map_err(|error| error.to_string())?;
    fs::create_dir_all(root.join("data")).map_err(|error| error.to_string())?;
    fs::write(
        logs.join("daemon.log"),
        "Authorization: Bearer obs-token-canary\npostgresql://obs:password@localhost/obs\nok line\n",
    )
    .map_err(|error| error.to_string())?;
    let state = state(database.url(), &root, &logs);
    let output = root.join("support.tar");
    let admission = state.admit_request().ok_or("admission unavailable")?;
    let bundle_state = state.clone();
    let bundle_output = output.clone();
    let runtime = tokio::runtime::Runtime::new().map_err(|error| error.to_string())?;
    let returned =
        runtime
            .block_on(admission.run_blocking(move || {
                crate::support::bundle::create(&bundle_state, &bundle_output)
            }))
            .map_err(|error| match error {
                crate::app::BlockingError::Deadline => "support bundle deadline".to_string(),
                crate::app::BlockingError::Join => "support bundle worker failed".to_string(),
            })??;
    drop(runtime);
    assert_eq!(returned["source"], "daemon-local");
    assert_eq!(
        fs::metadata(&output)
            .map_err(|error| error.to_string())?
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    let (names, members, modes) = archive_members(&output)?;
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
    assert!(modes.values().all(|mode| *mode == 0o600));
    let manifest: Value = serde_json::from_slice(
        members
            .get("manifest.json")
            .ok_or("support manifest missing")?,
    )
    .map_err(|error| error.to_string())?;
    assert!(manifest["createdAt"]
        .as_str()
        .is_some_and(|value| value.ends_with('Z')));
    for item in manifest["members"]
        .as_array()
        .ok_or("manifest members missing")?
    {
        let name = item["name"].as_str().ok_or("member name missing")?;
        let bytes = members.get(name).ok_or("manifest member absent")?;
        assert_eq!(item["bytes"], bytes.len());
        assert_eq!(item["sha256"], format!("{:x}", Sha256::digest(bytes)));
    }
    for bytes in members.values() {
        let text = String::from_utf8_lossy(bytes);
        assert!(!text.contains("obs-token-canary"));
        assert!(!text.contains("postgresql://obs:password"));
    }
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    Ok(())
}

fn archive_members(path: &Path) -> Result<ArchiveMembers, String> {
    let file = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut archive = tar::Archive::new(file);
    let mut names = Vec::new();
    let mut output = BTreeMap::new();
    let mut modes = BTreeMap::new();
    for entry in archive.entries().map_err(|error| error.to_string())? {
        let mut entry = entry.map_err(|error| error.to_string())?;
        let name = entry
            .path()
            .map_err(|error| error.to_string())?
            .to_string_lossy()
            .to_string();
        let mode = entry.header().mode().map_err(|error| error.to_string())?;
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        names.push(name.clone());
        modes.insert(name.clone(), mode);
        output.insert(name, bytes);
    }
    Ok((names, output, modes))
}

fn database() -> Result<Option<crate::test_database::TestDatabase>, String> {
    let Ok(url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(None);
    };
    crate::test_database::migrate(&url).map(Some)
}

fn unique_root() -> PathBuf {
    std::env::temp_dir().join(format!("lkjmc-obs-{}", Uuid::new_v4().simple()))
}

fn state(database_url: &str, root: &Path, logs: &Path) -> AppState {
    AppState::with_config_path(
        Some(database_url.into()),
        8,
        root.join("config").to_string_lossy().to_string(),
        logs.to_string_lossy().to_string(),
        root.join("jars").to_string_lossy().to_string(),
        root.join("data").to_string_lossy().to_string(),
        None,
        None,
        None,
    )
}
