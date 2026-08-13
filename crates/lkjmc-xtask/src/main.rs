#![forbid(unsafe_code)]

use std::env;
use std::process::Command;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [cmd, sub] if cmd == "quiet" && sub == "verify" => run_script("./scripts/verify-full.sh"),
        [] => Err(usage()),
        _ => Err(usage()),
    }
}

fn run_script(path: &str) -> Result<(), String> {
    let status = Command::new(path)
        .status()
        .map_err(|error| format!("failed to run {path}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{path} exited with {status}"))
    }
}

fn usage() -> String {
    "usage: lkjmc-xtask quiet verify".to_string()
}
