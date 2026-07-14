#![forbid(unsafe_code)]

mod args;
mod args_admin;
mod args_announcement;
mod args_asset;
mod args_bootstrap;
mod args_claim;
mod args_instance;
mod args_jar;
mod args_kit;
mod args_moderation;
mod args_network;
mod args_observability;
mod args_player;
mod args_security;
mod args_shop;
mod args_vote;
mod client;
mod commands;
mod commands_admin;
mod commands_announcement;
mod commands_asset;
mod commands_bootstrap;
mod commands_claim;
mod commands_db;
mod commands_instance;
mod commands_jar;
mod commands_kit;
mod commands_moderation;
mod commands_network;
mod commands_observability;
mod commands_player;
mod commands_security;
mod commands_shop;
mod commands_status;
mod commands_vote;
mod error;
mod format;

use std::env;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), error::CliError> {
    let args = args::parse(env::args().skip(1).collect())?;
    commands::run(args)
}
