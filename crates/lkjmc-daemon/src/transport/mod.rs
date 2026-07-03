use std::fs;
use std::path::Path;
use std::time::Duration;

use axum::Router;
use tokio::sync::oneshot;

use crate::app::AppState;

mod auth;
mod command;
mod routes;

pub fn serve(socket_path: &str, http_addr: Option<&str>, state: AppState) -> Result<(), String> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("start transport runtime: {error}"))?;
    runtime.block_on(serve_async(socket_path, http_addr, state))
}

async fn serve_async(
    socket_path: &str,
    http_addr: Option<&str>,
    state: AppState,
) -> Result<(), String> {
    let uds_listener = bind_uds(socket_path).await?;
    let (uds_stop_tx, uds_stop_rx) = oneshot::channel();
    let uds_state = state.clone();
    let uds_task = tokio::spawn(async move {
        axum::serve(uds_listener, uds_router(uds_state))
            .with_graceful_shutdown(shutdown_receiver(uds_stop_rx))
            .await
            .map_err(|error| format!("serve unix socket: {error}"))
    });

    let tcp = match http_addr {
        Some(addr) => Some(start_tcp(addr, state).await?),
        None => None,
    };
    wait_for_shutdown().await;
    let _ = uds_stop_tx.send(());
    if let Some((stop_tx, task)) = tcp {
        let _ = stop_tx.send(());
        join(task).await?;
    }
    join(uds_task).await
}

async fn start_tcp(
    addr: &str,
    state: AppState,
) -> Result<
    (
        oneshot::Sender<()>,
        tokio::task::JoinHandle<Result<(), String>>,
    ),
    String,
> {
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|error| format!("bind http {addr}: {error}"))?;
    let (stop_tx, stop_rx) = oneshot::channel();
    let task = tokio::spawn(async move {
        axum::serve(listener, tcp_router(state))
            .with_graceful_shutdown(shutdown_receiver(stop_rx))
            .await
            .map_err(|error| format!("serve http: {error}"))
    });
    Ok((stop_tx, task))
}

async fn bind_uds(path: &str) -> Result<tokio::net::UnixListener, String> {
    let value = Path::new(path);
    if value.exists() {
        fs::remove_file(value).map_err(|error| format!("remove socket {path}: {error}"))?;
    }
    tokio::net::UnixListener::bind(value).map_err(|error| format!("bind socket {path}: {error}"))
}

fn tcp_router(state: AppState) -> Router {
    routes::router(state, true)
}

fn uds_router(state: AppState) -> Router {
    routes::router(state, false)
}

async fn shutdown_receiver(receiver: oneshot::Receiver<()>) {
    let _ = receiver.await;
}

async fn wait_for_shutdown() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        if let Ok(mut terminate) = signal(SignalKind::terminate()) {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {},
                _ = terminate.recv() => {},
                _ = tokio::time::sleep(Duration::from_secs(u64::MAX)) => {},
            }
            return;
        }
    }
    let _ = tokio::signal::ctrl_c().await;
}

async fn join(task: tokio::task::JoinHandle<Result<(), String>>) -> Result<(), String> {
    task.await
        .map_err(|error| format!("transport task join: {error}"))?
}
