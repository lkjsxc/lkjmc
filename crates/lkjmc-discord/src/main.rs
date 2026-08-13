#![forbid(unsafe_code)]

mod commands;
mod config;
mod diagnostics;
mod discord_api;

use std::env;

use config::Config;

fn main() {
    if let Err(error) = run() {
        eprintln!("discord startup disabled: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = env::args().collect::<Vec<_>>();
    if args.get(1).map(String::as_str) == Some("--version") && args.len() == 2 {
        println!(
            "lkjmc-discord {} commit={} dirty={}",
            lkjmc_core::build_info::VERSION,
            lkjmc_core::build_info::COMMIT,
            lkjmc_core::build_info::dirty_label()
        );
        return Ok(());
    }
    let path = args.get(1).map(String::as_str).unwrap_or("discord.json");
    let config = Config::load(path).and_then(Config::validate)?;
    let diagnostics = diagnostics::Diagnostics::start();
    let _ = diagnostics.emit(
        lkjmc_core::observability::Outcome::Degraded,
        "commands-withdrawn",
    );
    if config.register_commands || args.iter().any(|arg| arg == "--register-commands") {
        config.validate_command_withdrawal()?;
        discord_api::register(&config, &commands::command_payload())?;
        let _ = diagnostics.emit(
            lkjmc_core::observability::Outcome::Degraded,
            "registration-withdrawn",
        );
    }
    if args.iter().any(|arg| arg == "--daemon-status") {
        return Err("Discord daemon status is withdrawn".into());
    }
    if args.iter().any(|arg| arg == "--check-config") {
        diagnostics.close();
        return Ok(());
    }
    diagnostics.close();
    Ok(())
}
