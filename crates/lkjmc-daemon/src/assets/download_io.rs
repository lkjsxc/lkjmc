use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;

use fs2::FileExt;
use md5::Md5;
use sha2::{Digest, Sha256, Sha512};
use uuid::Uuid;

const LOCK_ATTEMPTS: u32 = 200;
const LOCK_WAIT: Duration = Duration::from_millis(25);

#[derive(Debug)]
pub struct Hashes {
    pub md5: String,
    pub sha256: String,
    pub sha512: String,
    pub size_bytes: i64,
}

#[derive(Clone, Copy)]
pub enum ExpectedChecksum<'a> {
    Md5(&'a str),
    Sha256(&'a str),
    Sha512(&'a str),
}

pub fn download(
    url: &str,
    target: &Path,
    expected_size: Option<i64>,
    expected: ExpectedChecksum<'_>,
) -> Result<Hashes, String> {
    let parent = parent(target)?;
    fs::create_dir_all(parent).map_err(|error| format!("create download directory: {error}"))?;
    let _lock = target_lock(target)?;
    if target.exists() {
        let (hashes, size) = digest(File::open(target).map_err(|error| error.to_string())?, None)?;
        if let Ok(hashes) = verify(hashes, size, expected_size, expected) {
            return Ok(hashes);
        }
        fs::remove_file(target).map_err(|error| format!("remove invalid download: {error}"))?;
    }
    let temp = temporary_path(target)?;
    let result = download_temp(url, &temp).and_then(|(hashes, size)| {
        let hashes = verify(hashes, size, expected_size, expected)?;
        fs::rename(&temp, target)
            .map_err(|error| format!("atomically publish download: {error}"))?;
        sync_directory(parent)?;
        Ok(hashes)
    });
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

fn download_temp(url: &str, temp: &Path) -> Result<(Hashes, i64), String> {
    let response = ureq::get(url)
        .set("User-Agent", crate::commands::downloads::USER_AGENT)
        .call()
        .map_err(|_| "download request failed".to_string())?;
    let mut file =
        File::create(temp).map_err(|error| format!("create temporary download: {error}"))?;
    let result = digest(response.into_reader(), Some(&mut file));
    if result.is_ok() {
        file.sync_all()
            .map_err(|error| format!("sync temporary download: {error}"))?;
    }
    result
}

fn digest(mut reader: impl Read, mut file: Option<&mut File>) -> Result<(Hashes, i64), String> {
    let mut md5 = Md5::new();
    let mut sha256 = Sha256::new();
    let mut sha512 = Sha512::new();
    let mut size = 0_i64;
    let mut buffer = [0_u8; 8192];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|_| "read download bytes failed".to_string())?;
        if count == 0 {
            break;
        }
        size += i64::try_from(count).map_err(|error| error.to_string())?;
        md5.update(&buffer[..count]);
        sha256.update(&buffer[..count]);
        sha512.update(&buffer[..count]);
        if let Some(output) = file.as_deref_mut() {
            output
                .write_all(&buffer[..count])
                .map_err(|_| "write download bytes failed".to_string())?;
        }
    }
    Ok((
        Hashes {
            md5: format!("{:x}", md5.finalize()),
            sha256: format!("{:x}", sha256.finalize()),
            sha512: format!("{:x}", sha512.finalize()),
            size_bytes: size,
        },
        size,
    ))
}

fn verify(
    hashes: Hashes,
    size: i64,
    expected_size: Option<i64>,
    expected: ExpectedChecksum<'_>,
) -> Result<Hashes, String> {
    if expected_size.is_some_and(|value| value != size) {
        return Err(format!("download size mismatch: got {size}"));
    }
    let (actual, wanted) = match expected {
        ExpectedChecksum::Md5(value) => (&hashes.md5, value),
        ExpectedChecksum::Sha256(value) => (&hashes.sha256, value),
        ExpectedChecksum::Sha512(value) => (&hashes.sha512, value),
    };
    if actual.eq_ignore_ascii_case(wanted) {
        Ok(hashes)
    } else {
        Err("download checksum mismatch".to_string())
    }
}

fn target_lock(target: &Path) -> Result<File, String> {
    let lock = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(target.with_extension("lock"))
        .map_err(|error| format!("open download lock: {error}"))?;
    for _ in 0..LOCK_ATTEMPTS {
        if lock.try_lock_exclusive().is_ok() {
            return Ok(lock);
        }
        thread::sleep(LOCK_WAIT);
    }
    Err("download lock timed out".to_string())
}

fn temporary_path(target: &Path) -> Result<std::path::PathBuf, String> {
    let name = target
        .file_name()
        .ok_or_else(|| "download target has no file name".to_string())?;
    Ok(target.with_file_name(format!(
        ".{}.{}.part",
        name.to_string_lossy(),
        Uuid::new_v4()
    )))
}

fn parent(path: &Path) -> Result<&Path, String> {
    path.parent()
        .ok_or_else(|| format!("path has no parent: {}", path.display()))
}

fn sync_directory(path: &Path) -> Result<(), String> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|error| format!("sync download directory: {error}"))
}
