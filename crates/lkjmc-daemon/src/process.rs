use std::process::Command;

pub fn terminate_group(pid: u32) {
    let target = format!("-{pid}");
    if Command::new("kill")
        .arg("-TERM")
        .arg(&target)
        .status()
        .is_err()
    {
        eprintln!("failed to send TERM to process group {pid}");
    }
}

pub fn kill_group(pid: u32) {
    let target = format!("-{pid}");
    if Command::new("kill")
        .arg("-KILL")
        .arg(&target)
        .status()
        .is_err()
    {
        eprintln!("failed to send KILL to process group {pid}");
    }
}
