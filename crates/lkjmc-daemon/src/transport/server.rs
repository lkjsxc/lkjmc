use std::fs;
#[cfg(unix)]
use std::os::unix::fs::FileTypeExt;
use std::path::Path;
use std::time::Duration;

use axum::Router;
use tokio::sync::oneshot;

use crate::app::AppState;

use super::routes;

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
    let (uds_listener, peer_policy) = bind_uds(socket_path).await?;
    state.set_unix_peer_policy(peer_policy)?;
    let (uds_stop_tx, uds_stop_rx) = oneshot::channel();
    let uds_state = state.clone();
    let uds_task = tokio::spawn(async move {
        axum::serve(
            uds_listener,
            uds_router(uds_state).into_make_service_with_connect_info::<super::peer::UnixPeer>(),
        )
        .with_graceful_shutdown(shutdown_receiver(uds_stop_rx))
        .await
        .map_err(|error| format!("serve unix socket: {error}"))
    });

    let tcp = match http_addr {
        Some(addr) => Some(start_tcp(addr, state.clone()).await?),
        None => None,
    };
    wait_for_shutdown().await;
    state.stop_admission();
    let _ = uds_stop_tx.send(());
    if let Some((stop_tx, task)) = tcp {
        let _ = stop_tx.send(());
        join(task).await?;
    }
    join(uds_task).await?;
    state.wait_for_admitted_work().await;
    Ok(())
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
        axum::serve(
            listener,
            tcp_router(state).into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_receiver(stop_rx))
        .await
        .map_err(|error| format!("serve http: {error}"))
    });
    Ok((stop_tx, task))
}

async fn bind_uds(
    path: &str,
) -> Result<(tokio::net::UnixListener, super::peer::UnixPeerPolicy), String> {
    let value = Path::new(path);
    if value.exists() {
        let metadata = fs::symlink_metadata(value)
            .map_err(|error| format!("inspect socket {path}: {error}"))?;
        if !metadata.file_type().is_socket() {
            return Err(format!("refusing to remove non-socket path: {path}"));
        }
        match std::os::unix::net::UnixStream::connect(value) {
            Ok(_) => return Err(format!("daemon socket is already live: {path}")),
            Err(error) if error.kind() == std::io::ErrorKind::ConnectionRefused => {}
            Err(error) => return Err(format!("cannot verify daemon socket {path}: {error}")),
        }
        fs::remove_file(value).map_err(|error| format!("remove stale socket {path}: {error}"))?;
    }
    let listener = tokio::net::UnixListener::bind(value)
        .map_err(|error| format!("bind socket {path}: {error}"))?;
    std::fs::set_permissions(value, std::os::unix::fs::PermissionsExt::from_mode(0o660))
        .map_err(|_| "set socket permissions failed".to_string())?;
    let policy = super::peer::UnixPeerPolicy::from_socket(value)?;
    Ok((listener, policy))
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

#[cfg(test)]
mod tests {
    use super::bind_uds;

    #[tokio::test]
    async fn daemon_singleton_refuses_a_live_socket() -> Result<(), String> {
        let path = std::env::temp_dir().join(format!("lkjmc-daemon-{}.sock", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let listener =
            std::os::unix::net::UnixListener::bind(&path).map_err(|error| error.to_string())?;
        let result = bind_uds(path.to_str().ok_or("socket path is not UTF-8")?).await;
        drop(listener);
        std::fs::remove_file(&path).map_err(|error| error.to_string())?;
        assert!(result.is_err());
        Ok(())
    }
}
