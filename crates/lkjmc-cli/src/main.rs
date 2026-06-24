#![forbid(unsafe_code)]

mod args;
mod args_instance;
mod args_jar;
mod client;
mod commands;
mod commands_jar;
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
