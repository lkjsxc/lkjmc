#![forbid(unsafe_code)]

mod commands;
mod config;
mod daemon;
mod discord_api;
mod formatting;
mod interaction;
mod interaction_http;
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
        "ok discord config guilds={} channels={} commands={}",
        config.guild_allowlist.len(),
        config.channel_allowlist.len(),
        commands::command_payload()[0]["options"]
            .as_array()
            .map(Vec::len)
            .unwrap_or(0)
    );
    if config.register_commands || args.iter().any(|arg| arg == "--register-commands") {
        discord_api::register(&config, &commands::command_payload())?;
        println!("ok discord commands registered");
    }
    if args.iter().any(|arg| arg == "--daemon-status") {
        println!("ok daemon status {}", daemon::status(&config)?);
    }
    if let Some(addr) = config.interaction_bind.clone() {
        println!("ok discord interaction listener {addr}");
        interaction_http::serve(&addr, config)?;
    }
    Ok(())
}
