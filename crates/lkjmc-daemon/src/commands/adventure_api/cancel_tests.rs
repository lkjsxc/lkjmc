use std::os::unix::process::CommandExt;

use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use serde_json::json;
use uuid::Uuid;

use super::{cancellable, handle};
use crate::app::AppState;
use crate::runtime::local::LocalRuntime;

#[test]
fn fenced_runtime_cancellation_preserves_durable_session_and_refund() -> Result<(), String> {
    let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let mut guard = reset_and_migrate(&database_url)?;
    let mut command = std::process::Command::new("sleep");
    command.arg("5").process_group(0);
    let mut child = command.spawn().map_err(|error| error.to_string())?;
    let session_id = Uuid::new_v4();
    let instance_id = "fenced-cancel";
    let mut runtime = LocalRuntime::new();
    assert!(!runtime.recover(instance_id, child.id()).healthy);
    let state = state(database_url);
    state.set_runtime(Box::new(runtime))?;
    let result = (|| {
        lkjmc_store::instance::insert(
            &mut guard,
            instance_id,
            None,
            "folia",
            "stopped",
            &json!({}),
        )
        .map_err(|error| error.to_string())?;
        lkjmc_store::temporary::insert_instance(
            &mut guard,
            lkjmc_store::temporary::NewTemporaryInstance {
                instance_id,
                owner_kind: "adventure",
                owner_id: &session_id.to_string(),
                visibility: "hidden",
                world_path: "/tmp/fenced-cancel-world",
                server_port: 25577,
                max_lifetime_seconds: 300,
                retention_seconds: 0,
                cleanup_policy: "delete",
                lifecycle_state: "created",
                start_deadline_seconds: 60,
                metadata: json!({}),
            },
        )
        .map_err(|error| error.to_string())?;
        lkjmc_store::temporary::insert_session(
            &mut guard,
            lkjmc_store::temporary::NewAdventureSession {
                id: session_id,
                adventure_kind: "end-expedition",
                buyer_uuid: Uuid::new_v4(),
                buyer_name: "fenced-test",
                temporary_instance_id: instance_id,
                points_cost: 10,
                points_ledger_id: None,
                state: "pending",
                start_deadline_seconds: 60,
                stop_deadline_seconds: 60,
                metadata: json!({}),
            },
        )
        .map_err(|error| error.to_string())?;
        let response = handle(&state, request(session_id)?);
        assert!(!response.ok);
        assert_eq!(
            lkjmc_store::temporary::get_session(&mut guard, session_id)
                .map_err(|error| error.to_string())?
                .ok_or("session missing")?
                .state,
            "pending"
        );
        assert_eq!(
            lkjmc_store::temporary::get_instance(&mut guard, instance_id)
                .map_err(|error| error.to_string())?
                .ok_or("temporary instance missing")?
                .lifecycle_state,
            "created"
        );
        let refunds: i64 = guard
            .query_one("select count(*) from points_ledger", &[])
            .map_err(|error| error.to_string())?
            .get(0);
        assert_eq!(refunds, 0);
        Ok(())
    })();
    let _ = child.kill();
    let _ = child.wait();
    guard
        .batch_execute("select pg_advisory_unlock(752647)")
        .map_err(|error| error.to_string())?;
    result
}

#[test]
fn cancellation_only_allows_pre_active_sessions() {
    assert!(cancellable("ready"));
    assert!(!cancellable("active"));
}

fn request(session_id: Uuid) -> Result<CommandEnvelope, String> {
    Ok(CommandEnvelope {
        request_id: CommandId::parse("request id", "adventure-cancel")
            .map_err(|error| error.to_string())?,
        actor: Actor {
            kind: ActorKind::Cli,
            name: "fenced-test".to_string(),
        },
        command: "adventure.session.cancel".to_string(),
        body: json!({"sessionId":session_id.to_string(),"reason":"operator cancel"}),
    })
}

fn state(database_url: String) -> AppState {
    AppState::with_config_path(
        Some(database_url),
        2,
        "/tmp/config".to_string(),
        "/tmp/log".to_string(),
        "/tmp/jars".to_string(),
        "/tmp/data".to_string(),
        None,
        None,
        None,
    )
}

fn reset_and_migrate(database_url: &str) -> Result<postgres::Client, String> {
    let mut client =
        lkjmc_store::pool::connect_single(database_url).map_err(|error| error.to_string())?;
    client
        .batch_execute(
            "select pg_advisory_lock(752647); drop schema public cascade; create schema public",
        )
        .map_err(|error| error.to_string())?;
    lkjmc_store::migrate::apply(&mut client).map_err(|error| error.to_string())?;
    Ok(client)
}
