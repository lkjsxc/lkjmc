#![forbid(unsafe_code)]
mod admin_api;
mod adventure_api;
mod announcement_api;
mod api;
#[cfg(test)]
mod api_tests;
mod app;
mod asset_api;
mod audit_helpers;
mod authz;
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
mod downloads_versions;
mod http_api;
mod http_auth;
mod instance_api;
mod instance_heartbeat;
mod instance_helpers;
mod instance_launch;
mod instance_lifecycle;
mod instance_read;
mod instance_wake_join;
mod instance_wake_runtime;
mod jar_prune;
mod jars;
mod logs;
#[cfg(test)]
mod menu_response_shapes;
mod player_achievements_api;
mod player_api;
mod player_daily_api;
mod player_exchange_api;
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
mod purpur_downloads;
mod rcon;
mod reconciler;
mod reconciler_policy;
mod runtime;
mod runtime_kubernetes;
mod runtime_local;
mod runtime_local_adapter;
mod security_api;
mod security_token;
mod socket_api;
mod status_api;
mod templates;
mod temporary_api;
mod temporary_cleanup;
mod web_api;
#[cfg(test)]
mod web_api_tests;
mod web_auth;
mod web_html;
mod web_request;
mod web_sessions;

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
        args.http_token_file,
        args.http_token,
    );
    configure_runtime(&state)?;
    let reconciler_enabled = state.database_url().is_some();
    state.with_runtime_metadata(args.socket.clone(), args.http.clone(), reconciler_enabled)?;
    reconciler::recover(&state)?;
    if reconciler_enabled {
        let reconcile_state = state.clone();
        let _reconciler = reconciler::start_loop(reconcile_state);
        let cleanup_state = state.clone();
        let _temporary_cleanup = temporary_cleanup::start_loop(cleanup_state);
    }
    if let Some(http_addr) = args.http {
        let http_state = state.clone();
        thread::spawn(move || {
            if let Err(error) = http_api::serve(&http_addr, http_state) {
                eprintln!("{error}");
            }
        });
    }
    socket_api::serve(&args.socket, state)
}

fn configure_runtime(state: &AppState) -> Result<(), String> {
    let Some(config) = state.runtime_config()? else {
        return Ok(());
    };
    match config.runtime.adapter {
        lkjmc_core::config::RuntimeAdapter::LocalProcess => Ok(()),
        lkjmc_core::config::RuntimeAdapter::Kubernetes => {
            let kubernetes = config
                .runtime
                .kubernetes
                .ok_or_else(|| "runtime.kubernetes missing".to_string())?;
            state.set_runtime(Box::new(runtime_kubernetes::KubernetesRuntime::new(
                kubernetes,
            )))
        }
    }
}
