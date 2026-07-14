#[allow(dead_code)]
mod support;

use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope, CommandResponse};
use lkjmc_core::id::CommandId;
use lkjmc_store::{migrate, observability};
use serde_json::json;
use uuid::Uuid;

#[test]
fn migration_persists_queries_and_retain_events() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    assert!(migrate::apply(client)?.contains(&51));
    let operation_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    let request = request("obs-persistence", operation_id, correlation_id);
    let response = CommandResponse {
        request_id: request.request_id.clone(),
        ok: true,
        body: Some(json!({"daemon":"running"})),
        error: None,
    };
    let event = observability::record_command(client, &request, &response)?;
    assert_eq!(event.operation_id, Some(operation_id));
    assert_eq!(event.correlation_id, Some(correlation_id));
    for query in [
        observability::EventQuery {
            request_id: Some("obs-persistence"),
            operation_id: None,
            correlation_id: None,
            limit: 500,
        },
        observability::EventQuery {
            request_id: None,
            operation_id: Some(operation_id),
            correlation_id: None,
            limit: 500,
        },
        observability::EventQuery {
            request_id: None,
            operation_id: None,
            correlation_id: Some(correlation_id),
            limit: 500,
        },
    ] {
        let rows = observability::query(client, query)?;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["source"], "daemon-local");
    }
    client.execute(
        "update observability_events set occurred_at=now()-interval '15 days' where request_id=$1",
        &[&"obs-persistence"],
    )?;
    assert_eq!(observability::retain(client)?, 1);
    assert!(observability::query(
        client,
        observability::EventQuery {
            request_id: Some("obs-persistence"),
            operation_id: None,
            correlation_id: None,
            limit: 500,
        }
    )?
    .is_empty());
    Ok(())
}

#[test]
fn reused_request_id_records_each_postgresql_attempt() -> Result<(), lkjmc_store::error::StoreError>
{
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    let request_id = "obs-reused-request";
    let mut first = request(request_id, Uuid::new_v4(), Uuid::new_v4());
    first.command = "status".into();
    let mut second = request(request_id, Uuid::new_v4(), Uuid::new_v4());
    second.command = "doctor".into();
    observability::record_command(client, &first, &success(&first))?;
    observability::record_command(client, &second, &success(&second))?;
    let events = observability::query(
        client,
        observability::EventQuery {
            request_id: Some(request_id),
            operation_id: None,
            correlation_id: None,
            limit: 10,
        },
    )?;
    assert_eq!(events.len(), 2);
    assert_ne!(events[0]["eventId"], events[1]["eventId"]);
    assert_ne!(events[0]["operationId"], events[1]["operationId"]);
    Ok(())
}

#[test]
fn malicious_attributes_retain_redacted_postgresql_event(
) -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    for _ in 0..2 {
        let mut command = request("obs-malicious-attribute", Uuid::new_v4(), Uuid::new_v4());
        command.command = "Bearer obs-token-canary".into();
        command.actor.name = "https://user:password@example.test".into();
        observability::record_command(client, &command, &success(&command))?;
    }
    let events = observability::query(
        client,
        observability::EventQuery {
            request_id: Some("obs-malicious-attribute"),
            operation_id: None,
            correlation_id: None,
            limit: 10,
        },
    )?;
    assert_eq!(events.len(), 2);
    for event in events {
        assert_eq!(event["attributes"]["redacted"], true);
        assert_eq!(event["actorName"], "[redacted]");
        assert!(!event.to_string().contains("obs-token-canary"));
    }
    Ok(())
}

fn success(request: &CommandEnvelope) -> CommandResponse {
    CommandResponse {
        request_id: request.request_id.clone(),
        ok: true,
        body: Some(json!({"daemon":"running"})),
        error: None,
    }
}

fn request(name: &str, operation_id: Uuid, correlation_id: Uuid) -> CommandEnvelope {
    CommandEnvelope {
        request_id: CommandId::parse("request id", name.to_string())
            .unwrap_or_else(|_| CommandId::internal("observability-test")),
        actor: Actor {
            kind: ActorKind::Cli,
            name: "test-operator".into(),
        },
        command: "status".into(),
        body: json!({
            "operationId": operation_id,
            "correlationId": correlation_id
        }),
    }
}
