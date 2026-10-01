use crate::{client::file_hash, state::Store};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{fs::File, path::Path};
use uuid::Uuid;

pub fn intent(
    store: &Store,
    backup: Uuid,
    job: Uuid,
    server: Uuid,
    manifest: &Value,
) -> Result<bool> {
    let Some(record) = store.read::<Value>("prunes", backup)? else {
        return Ok(false);
    };
    ensure!(
        record == json!({"job_id":job,"server_id":server,"manifest":manifest}),
        "Backup deletion journal disagrees with Core; reconcile before deleting files"
    );
    Ok(true)
}

pub async fn prepare(
    store: &Store,
    backup: Uuid,
    job: Uuid,
    server: Uuid,
    manifest: &Value,
) -> Result<()> {
    if intent(store, backup, job, server, manifest)? {
        return Ok(());
    }
    ensure!(
        manifest["backup_id"] == backup.to_string()
            && manifest["world"]["server_id"] == server.to_string()
            && manifest["verified"] == true
            && manifest["world"]["snapshot"] == format!("b-{backup}"),
        "Backup identity does not match the deletion target"
    );
    let root = store.root.join("backups");
    verify_file(&root.join(format!("{backup}.tar.gz")), &manifest["world"]).await?;
    verify_file(&root.join(format!("{backup}.dump")), &manifest["database"]).await?;
    let checkpoint = root.join(format!("{backup}.json"));
    regular(&checkpoint)?;
    let checkpoint: Value = serde_json::from_slice(&std::fs::read(checkpoint)?)?;
    ensure!(
        checkpoint["result"] == *manifest,
        "Stored backup receipt disagrees with Core"
    );
    store.write(
        "prunes",
        backup,
        &json!({"job_id":job,"server_id":server,"manifest":manifest}),
    )?;
    Ok(())
}

async fn verify_file(path: &Path, manifest: &Value) -> Result<()> {
    regular(path)?;
    ensure!(
        Some(std::fs::metadata(path)?.len()) == manifest["bytes"].as_u64()
            && file_hash(path).await?
                == manifest["sha256"]
                    .as_str()
                    .context("Backup checksum is absent")?,
        "Backup bytes differ from the saved manifest"
    );
    Ok(())
}

fn regular(path: &Path) -> Result<()> {
    ensure!(
        std::fs::symlink_metadata(path)?.is_file(),
        "Backup artifact must be a regular file, without symlinks"
    );
    Ok(())
}

/// Incus snapshot deletion precedes file removal. Missing files are acceptable
/// only after the durable intent exists, so a crash resumes the same deletion.
pub fn remove_files(
    store: &Store,
    backup: Uuid,
    job: Uuid,
    server: Uuid,
    manifest: &Value,
) -> Result<()> {
    ensure!(
        intent(store, backup, job, server, manifest)?,
        "Refusing backup deletion without durable intent"
    );
    let root = store.root.join("backups");
    for extension in ["tar.gz", "dump", "json"] {
        let path = root.join(format!("{backup}.{extension}"));
        match std::fs::symlink_metadata(&path) {
            Ok(meta) => {
                ensure!(
                    meta.is_file(),
                    "Backup artifact was replaced with a non-regular file"
                );
                std::fs::remove_file(path)?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    File::open(root)?.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    async fn fixture() -> (Store, Uuid, Uuid, Uuid, Value) {
        let store =
            Store::open(&std::env::temp_dir().join(format!("lkjmc-retention-{}", Uuid::new_v4())))
                .unwrap();
        let backup = Uuid::new_v4();
        let job = Uuid::new_v4();
        let server = Uuid::new_v4();
        let bytes = b"retention fixture, not an Incus export";
        let hash = hex::encode(Sha256::digest(bytes));
        let manifest = json!({"backup_id":backup,"verified":true,"world":{"server_id":server,"snapshot":format!("b-{backup}"),"bytes":bytes.len(),"sha256":hash},"database":{"bytes":bytes.len(),"sha256":hash}});
        for ext in ["tar.gz", "dump"] {
            std::fs::write(
                store.root.join("backups").join(format!("{backup}.{ext}")),
                bytes,
            )
            .unwrap();
        }
        store
            .write("backups", backup, &json!({"resume":true,"result":manifest}))
            .unwrap();
        (store, backup, job, server, manifest)
    }

    #[tokio::test]
    async fn deletion_resumes_after_each_file_boundary_without_touching_another_backup() {
        for cut in 0..=3 {
            let (store, backup, job, server, manifest) = fixture().await;
            let other = store
                .root
                .join("backups")
                .join(format!("{}.dump", Uuid::new_v4()));
            std::fs::write(&other, b"keep this backup").unwrap();
            assert!(remove_files(&store, backup, job, server, &manifest).is_err());
            prepare(&store, backup, job, server, &manifest)
                .await
                .unwrap();
            // Simulate the durable filesystem state at each interrupted unlink.
            for ext in ["tar.gz", "dump", "json"].iter().take(cut) {
                std::fs::remove_file(store.root.join("backups").join(format!("{backup}.{ext}")))
                    .unwrap();
            }
            prepare(&store, backup, job, server, &manifest)
                .await
                .unwrap();
            remove_files(&store, backup, job, server, &manifest).unwrap();
            remove_files(&store, backup, job, server, &manifest).unwrap();
            assert_eq!(std::fs::read(&other).unwrap(), b"keep this backup");
            assert!(intent(&store, backup, Uuid::new_v4(), server, &manifest).is_err());
            std::fs::remove_dir_all(&store.root).unwrap();
        }
    }

    #[tokio::test]
    async fn corrupt_and_symlinked_backups_are_not_deleted() {
        let (store, backup, job, server, manifest) = fixture().await;
        let archive = store.root.join("backups").join(format!("{backup}.tar.gz"));
        std::fs::write(&archive, b"corrupted").unwrap();
        assert!(
            prepare(&store, backup, job, server, &manifest)
                .await
                .is_err()
        );
        assert!(archive.exists());
        let target = store.root.join("foreign-file");
        std::fs::write(&target, b"foreign data").unwrap();
        std::fs::remove_file(&archive).unwrap();
        std::os::unix::fs::symlink(&target, &archive).unwrap();
        assert!(
            prepare(&store, backup, job, server, &manifest)
                .await
                .is_err()
        );
        assert!(!intent(&store, backup, job, server, &manifest).unwrap());
        assert_eq!(std::fs::read(target).unwrap(), b"foreign data");
        std::fs::remove_dir_all(&store.root).unwrap();
    }
}
