use std::process::Command;

pub fn terminate_group(pid: u32) {
    if !signal_group(pid, "-TERM") {
        eprintln!("failed to send TERM to process group {pid}");
    }
}

pub fn kill_group(pid: u32) {
    if !signal_group(pid, "-KILL") {
        eprintln!("failed to send KILL to process group {pid}");
    }
}

pub fn group_exists(pid: u32) -> bool {
    signal_group(pid, "-0")
}

fn signal_group(pid: u32, signal: &str) -> bool {
    let target = format!("-{pid}");
    Command::new("kill")
        .arg(signal)
        .arg(&target)
        .status()
        .is_ok_and(|status| status.success())
}
