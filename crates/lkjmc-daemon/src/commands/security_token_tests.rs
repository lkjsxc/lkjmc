use std::fs;

use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use serde_json::json;
use tokio::sync::oneshot;

use super::*;

#[test]
fn generated_token_is_secret_shaped() {
    let token = generate_token();
    assert!(token.len() >= 40);
    assert!(!token.contains('\n'));
}

#[tokio::test]
async fn rotation_probes_the_live_transport_before_retiring_old_token() -> Result<(), String> {
    let path = std::env::temp_dir().join(format!("lkjmc-rotate-{}.token", std::process::id()));
    fs::write(&path, "old-token\n").map_err(|error| error.to_string())?;
    let state = AppState::with_config_path(
        None,
        8,
        "/c".into(),
        "/l".into(),
        "/j".into(),
        "/d".into(),
        None,
        Some(path.to_string_lossy().to_string()),
        Some("old-token".into()),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|error| error.to_string())?;
    let address = listener
        .local_addr()
        .map_err(|error| error.to_string())?
        .to_string();
    state.with_runtime_metadata("/tmp/lkjmc-test.sock".into(), Some(address), false)?;
    let (stop, stopped) = oneshot::channel();
    let server_state = state.clone();
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            crate::transport::routes::router(server_state, true),
        )
        .with_graceful_shutdown(async {
            let _ = stopped.await;
        })
        .await
        .map_err(|error| error.to_string())
    });
    let request = request("security.daemon-token.rotate")?;
    let rotate_state = state.clone();
    let response = tokio::task::spawn_blocking(move || rotate(&rotate_state, request))
        .await
        .map_err(|error| error.to_string())?;
    let _ = stop.send(());
    server.await.map_err(|error| error.to_string())??;
    assert!(response.ok);
    let new_token = state.http_token().ok_or("missing new token")?;
    assert_ne!(new_token, "old-token");
    assert!(state.http_previous_token().is_none());
    assert_eq!(
        fs::read_to_string(&path)
            .map_err(|error| error.to_string())?
            .trim(),
        new_token
    );
    fs::remove_file(path).ok();
    Ok(())
}

fn request(command: &str) -> Result<CommandEnvelope, String> {
    Ok(CommandEnvelope {
        request_id: CommandId::parse("request id", command).map_err(|error| error.to_string())?,
        actor: Actor {
            kind: ActorKind::Cli,
            name: "test".into(),
        },
        command: command.into(),
        body: json!({}),
    })
}
