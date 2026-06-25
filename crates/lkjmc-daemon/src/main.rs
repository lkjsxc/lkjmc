#![forbid(unsafe_code)]

mod announcement_api;
mod api;
mod app;
mod audit_helpers;
mod config_api;
mod daemon_config;
mod downloads;
mod downloads_io;
mod http_api;
mod instance_api;
mod instance_heartbeat;
mod instance_helpers;
mod instance_lifecycle;
mod instance_read;
mod jar_prune;
mod jars;
mod logs;
mod player_achievements_api;
mod player_api;
mod player_daily_api;
mod player_homes_api;
mod player_kit_api;
mod player_mail_api;
mod player_moderation_api;
mod player_note_api;
mod player_party_api;
mod player_points_api;
mod player_report_api;
mod player_restore_api;
mod player_session_api;
mod player_settings_api;
mod player_shop_api;
mod player_teleport_api;
mod player_warning_api;
mod player_warps_api;
mod process;
mod rcon;
mod reconciler;
mod runtime;
mod runtime_local;
mod socket_api;
mod templates;

use std::env;
use std::thread;

use app::AppState;

#[derive(Debug, Clone)]
struct DaemonArgs {
    socket: String,
    http: Option<String>,
    http_token: Option<String>,
    database_url: Option<String>,
    config_root: String,
    log_root: String,
    jar_root: String,
    data_root: String,
    config_path: Option<String>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = parse_args(env::args().skip(1).collect())?;
    let state = AppState::with_config_path(
        args.database_url,
        args.config_root,
        args.log_root,
        args.jar_root,
        args.data_root,
        args.config_path,
    );
    reconciler::recover(&state)?;
    if state.database_url().is_some() {
        let reconcile_state = state.clone();
        let _reconciler = reconciler::start_loop(reconcile_state);
    }
    if let Some(http_addr) = args.http {
        let http_state = state.clone();
        let http_token = args.http_token.clone();
        thread::spawn(move || {
            if let Err(error) = http_api::serve(&http_addr, http_state, http_token) {
                eprintln!("{error}");
            }
        });
    }
    socket_api::serve(&args.socket, state)
}

fn parse_args(values: Vec<String>) -> Result<DaemonArgs, String> {
    let mut socket = "/run/lkjmc/daemon.sock".to_string();
    let mut http = Some("127.0.0.1:8765".to_string());
    let mut http_token = None;
    let mut database_url = env::var("LKJMC_DATABASE_URL").ok();
    let mut config_root = "/etc/lkjmc".to_string();
    let mut log_root = "/var/log/lkjmc/instances".to_string();
    let mut jar_root = "/opt/lkjmc/jars".to_string();
    let mut data_root = "/var/lib/lkjmc/instances".to_string();
    let config_path = requested_config(&values)?;
    if let Some(config_path) = &config_path {
        let config = daemon_config::load(config_path)?;
        socket = config.socket;
        database_url = Some(config.database_url);
        config_root = config.config_root;
        log_root = config.log_root;
        jar_root = config.jar_root;
        data_root = config.data_root;
    }
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--socket" => {
                socket = value_after(&values, index, "--socket")?;
                index += 2;
            }
            "--config" => {
                let _ = value_after(&values, index, "--config")?;
                index += 2;
            }
            "--config-root" => {
                config_root = value_after(&values, index, "--config-root")?;
                index += 2;
            }
            "--http" => {
                let value = value_after(&values, index, "--http")?;
                http = (value != "none").then_some(value);
                index += 2;
            }
            "--http-token" => {
                http_token = Some(value_after(&values, index, "--http-token")?);
                index += 2;
            }
            "--database-url" => {
                database_url = Some(value_after(&values, index, "--database-url")?);
                index += 2;
            }
            "--log-root" => {
                log_root = value_after(&values, index, "--log-root")?;
                index += 2;
            }
            "--jar-root" => {
                jar_root = value_after(&values, index, "--jar-root")?;
                index += 2;
            }
            "--data-root" => {
                data_root = value_after(&values, index, "--data-root")?;
                index += 2;
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(DaemonArgs {
        socket,
        http,
        http_token,
        database_url,
        config_root,
        log_root,
        jar_root,
        data_root,
        config_path,
    })
}

fn requested_config(values: &[String]) -> Result<Option<String>, String> {
    for (index, value) in values.iter().enumerate() {
        if value == "--config" {
            return value_after(values, index, "--config").map(Some);
        }
    }
    Ok(daemon_config::default_path())
}

fn value_after(values: &[String], index: usize, flag: &str) -> Result<String, String> {
    values
        .get(index + 1)
        .cloned()
        .ok_or_else(|| format!("missing value for {flag}"))
}
