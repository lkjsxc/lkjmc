#![forbid(unsafe_code)]
mod app;
mod assets;
mod authz;
mod command_lifecycle;
mod commands;
mod credential_cache;
mod dispatch;
#[cfg(test)]
mod fault_harness;
mod runtime;
mod security_audit;
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
    let state = configure_runtime(state)?;
    state.with_runtime_metadata(args.socket.clone(), args.http.clone(), false)?;
    transport::serve(&args.socket, args.http.as_deref(), state)
}

fn configure_runtime(state: AppState) -> Result<AppState, String> {
    let Some(config) = state.runtime_config()? else {
        return Ok(state);
    };
    match config.runtime.adapter {
        lkjmc_core::config::RuntimeAdapter::LocalProcess => Ok(state),
        lkjmc_core::config::RuntimeAdapter::Kubernetes => {
            let kubernetes = config
                .runtime
                .kubernetes
                .ok_or_else(|| "runtime.kubernetes missing".to_string())?;
            state.with_runtime(std::sync::Arc::new(
                runtime::kubernetes::KubernetesRuntime::new(kubernetes),
            ))
        }
    }
}
