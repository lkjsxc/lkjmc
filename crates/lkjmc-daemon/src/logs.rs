use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use lkjmc_core::validation::is_kebab_id;

pub fn tail(log_root: &str, instance_id: &str, lines: usize) -> Result<Vec<String>, String> {
    if !is_kebab_id(instance_id) {
        return Err("invalid instance id".to_string());
    }
    let path = Path::new(log_root).join(instance_id).join("current.log");
    let mut file = File::open(&path).map_err(|error| format!("open log: {error}"))?;
    let size = file
        .seek(SeekFrom::End(0))
        .map_err(|error| format!("seek log: {error}"))?;
    let read_bytes = size.min(65_536);
    file.seek(SeekFrom::End(-(read_bytes as i64)))
        .map_err(|error| format!("seek tail: {error}"))?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)
        .map_err(|error| format!("read log: {error}"))?;
    let mut values: Vec<String> = buffer.lines().map(ToString::to_string).collect();
    if values.len() > lines {
        values = values.split_off(values.len() - lines);
    }
    Ok(values)
}
