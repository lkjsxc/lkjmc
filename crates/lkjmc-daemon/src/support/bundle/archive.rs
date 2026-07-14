use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;
use std::time::Duration;

use uuid::Uuid;

use super::Deadline;

mod output;
pub(super) use output::{canonical_directory, VerifiedOutput};

pub(super) fn write(
    output: &VerifiedOutput,
    entries: &[(String, Vec<u8>)],
    deadline: Deadline,
    fault_delay: Duration,
) -> Result<(), String> {
    deadline.check()?;
    let temp_name = OsString::from(format!(".lkjmc-support-{}.tmp", Uuid::new_v4().simple()));
    let temp = output.child(&temp_name);
    let result = write_inner(output, &temp_name, &temp, entries, deadline, fault_delay);
    if result.is_err() {
        cleanup(output, &temp);
    }
    result
}

fn write_inner(
    output: &VerifiedOutput,
    temp_name: &OsStr,
    temp: &Path,
    entries: &[(String, Vec<u8>)],
    deadline: Deadline,
    fault_delay: Duration,
) -> Result<(), String> {
    deadline.check()?;
    let file = OpenOptions::new()
        .write(true)
        .read(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(temp)
        .map_err(|_| "create private support archive failed")?;
    if !file
        .metadata()
        .map_err(|_| "inspect support archive failed")?
        .is_file()
    {
        return Err("support archive temporary is not regular".into());
    }
    cooperative_delay(fault_delay, deadline)?;
    let mut archive = tar::Builder::new(file);
    for (name, value) in entries {
        deadline.check()?;
        if crate::support::redaction::contains_sensitive_canary(value) {
            return Err("support archive secret canary detected".into());
        }
        append(&mut archive, name, value)?;
    }
    deadline.check()?;
    archive
        .finish()
        .map_err(|_| "finish support archive failed")?;
    let mut file = archive
        .into_inner()
        .map_err(|_| "close support archive failed")?;
    deadline.check()?;
    file.flush().map_err(|_| "flush support archive failed")?;
    deadline.check()?;
    file.sync_all().map_err(|_| "sync support archive failed")?;
    deadline.check()?;
    final_scan(&mut file, deadline)?;
    deadline.check()?;
    fs::hard_link(temp, output.target()).map_err(|_| "publish support archive failed")?;
    deadline.check()?;
    output
        .directory
        .sync_all()
        .map_err(|_| "sync support archive directory failed")?;
    deadline.check()?;
    fs::remove_file(output.child(temp_name))
        .map_err(|_| "remove support archive temporary failed")?;
    Ok(())
}

fn append(archive: &mut tar::Builder<File>, name: &str, value: &[u8]) -> Result<(), String> {
    let mut header = tar::Header::new_gnu();
    header.set_size(value.len() as u64);
    header.set_mode(0o600);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_cksum();
    archive
        .append_data(&mut header, name, value)
        .map_err(|_| "write support member failed".to_string())
}

fn final_scan(file: &mut File, deadline: Deadline) -> Result<(), String> {
    file.seek(SeekFrom::Start(0))
        .map_err(|_| "scan support archive failed")?;
    let mut retained = Vec::new();
    let mut chunk = [0_u8; 16 * 1024];
    loop {
        deadline.check()?;
        let count = file
            .read(&mut chunk)
            .map_err(|_| "scan support archive failed")?;
        if count == 0 {
            break;
        }
        retained.extend_from_slice(&chunk[..count]);
    }
    if crate::support::redaction::contains_sensitive_canary(&retained) {
        return Err("support archive final canary scan failed".into());
    }
    Ok(())
}

fn cooperative_delay(mut delay: Duration, deadline: Deadline) -> Result<(), String> {
    while !delay.is_zero() {
        let remaining = deadline.remaining()?;
        let slice = delay.min(remaining).min(Duration::from_millis(5));
        std::thread::sleep(slice);
        delay = delay.saturating_sub(slice);
    }
    deadline.check()
}

fn cleanup(output: &VerifiedOutput, temp: &Path) {
    let temp_id = fs::metadata(temp)
        .ok()
        .map(|value| (value.dev(), value.ino()));
    if let (Some(id), Ok(target)) = (temp_id, fs::metadata(output.target())) {
        if id == (target.dev(), target.ino()) {
            let _ = fs::remove_file(output.target());
        }
    }
    let _ = fs::remove_file(temp);
}
