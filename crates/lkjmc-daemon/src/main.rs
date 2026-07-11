#![forbid(unsafe_code)]
mod app;
mod assets;
mod authz;
mod commands;
mod dispatch;
#[cfg(test)]
mod fault_harness;
mod reconcile;
mod runtime;
mod support;
mod templates;
#[cfg(test)]
mod test_database;
#[cfg(test)]
mod tests;
mod transport;
mod web;

use app::AppState;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = support::daemon_args::parse(std::env::args().skip(1).collect())?;
    let state = AppState::with_config_path(
        args.database_url,
        args.database_pool_size,
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
    reconcile::reconciler::recover(&state)?;
    if reconciler_enabled {
        let reconcile_state = state.clone();
        let _reconciler = reconcile::reconciler::start_loop(reconcile_state);
        let cleanup_state = state.clone();
        let _temporary_cleanup = reconcile::temporary_cleanup::start_loop(cleanup_state);
    }
    transport::serve(&args.socket, args.http.as_deref(), state)
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
            state.set_runtime(Box::new(runtime::kubernetes::KubernetesRuntime::new(
                kubernetes,
            )))
        }
    }
}
