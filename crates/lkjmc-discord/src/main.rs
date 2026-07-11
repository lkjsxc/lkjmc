#![forbid(unsafe_code)]

mod commands;
mod config;
mod discord_api;
mod interaction;
mod interaction_server;
mod signature;

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
    let path = args.get(1).map(String::as_str).unwrap_or("discord.json");
    let config = Config::load(path).and_then(Config::validate)?;
    println!(
        "ok discord config guilds={} commands={}",
        config.guild_allowlist.len(),
        commands::command_payload()
            .as_array()
            .map(Vec::len)
            .unwrap_or(0)
    );
    if config.register_commands || args.iter().any(|arg| arg == "--register-commands") {
        config.validate_command_withdrawal()?;
        discord_api::register(&config, &commands::command_payload())?;
        println!("ok discord commands withdrawn");
    }
    if args.iter().any(|arg| arg == "--daemon-status") {
        return Err("Discord daemon status is withdrawn".into());
    }
    if args.iter().any(|arg| arg == "--check-config") {
        return Ok(());
    }
    if let Some(addr) = config.interaction_bind.clone() {
        println!("ok discord interaction listener {addr}");
        interaction_server::serve(&addr, config)?;
    }
    Ok(())
}
