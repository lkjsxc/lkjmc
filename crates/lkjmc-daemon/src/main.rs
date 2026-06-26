#![forbid(unsafe_code)]
mod announcement_api;
mod api;
mod app;
mod asset_api;
mod audit_helpers;
mod bootstrap_api;
mod bootstrap_facts;
mod claim_api;
mod claim_create;
mod claim_read;
mod claim_trust;
mod config_api;
mod daemon_args;
mod daemon_config;
mod doctor_api;
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
mod player_vote_api;
mod player_warning_api;
mod player_warps_api;
mod plugin_assets;
mod plugin_downloads;
mod plugin_install;
mod process;
mod rcon;
mod reconciler;
mod runtime;
mod runtime_local;
mod socket_api;
mod status_api;
mod templates;

use std::thread;

use app::AppState;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = daemon_args::parse(std::env::args().skip(1).collect())?;
    let state = AppState::with_config_path(
        args.database_url,
        args.config_root,
        args.log_root,
        args.jar_root,
        args.data_root,
        args.config_path,
    );
    let reconciler_enabled = state.database_url().is_some();
    state.with_runtime_metadata(args.socket.clone(), args.http.clone(), reconciler_enabled)?;
    reconciler::recover(&state)?;
    if reconciler_enabled {
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
