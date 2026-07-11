use std::process::{Command, Stdio};

pub fn terminate_group(pid: u32) -> bool {
    signal_group(pid, "-TERM")
}

pub fn kill_group(pid: u32) -> bool {
    signal_group(pid, "-KILL")
}

pub fn group_exists(pid: u32) -> bool {
    signal_group(pid, "-0")
}

fn signal_group(pid: u32, signal: &str) -> bool {
    let target = format!("-{pid}");
    Command::new("kill")
        .arg(signal)
        .arg("--")
        .arg(&target)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}
