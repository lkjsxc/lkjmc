#![forbid(unsafe_code)]

mod args;
mod args_announcement;
mod args_instance;
mod args_jar;
mod args_moderation;
mod args_player;
mod args_shop;
mod client;
mod commands;
mod commands_announcement;
mod commands_db;
mod commands_instance;
mod commands_jar;
mod commands_moderation;
mod commands_player;
mod commands_shop;
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
