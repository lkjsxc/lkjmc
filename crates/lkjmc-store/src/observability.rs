use std::cell::Cell;
use std::collections::BTreeMap;

use lkjmc_core::command::{ActorKind, CommandEnvelope, CommandResponse};
use lkjmc_core::observability::{
    correlation_ids, Component, EventEnvelope, EventKind, Outcome, Severity, Surface,
};
use postgres::GenericClient;
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

thread_local! {
    static EXECUTION_EVENT_ID: Cell<Option<Uuid>> = const { Cell::new(None) };
}

mod query;
pub use query::{query, retain, EventQuery};

pub fn with_command_execution<T>(action: impl FnOnce() -> T) -> T {
    EXECUTION_EVENT_ID.with(|current| {
        let previous = current.replace(Some(Uuid::new_v4()));
        let result = action();
        current.set(previous);
        result
    })
}

pub fn record_command<C: GenericClient>(
    client: &mut C,
    request: &CommandEnvelope,
    response: &CommandResponse,
) -> Result<EventEnvelope, StoreError> {
    let event = command_event(request, response)?;
    record_command_event(client, &event)?;
    Ok(event)
}

pub fn record_command_event<C: GenericClient>(
    client: &mut C,
    event: &EventEnvelope,
) -> Result<(), StoreError> {
    upsert_operation(client, event)?;
    insert_event(client, event)
}

pub fn command_event(
    request: &CommandEnvelope,
    response: &CommandResponse,
) -> Result<EventEnvelope, StoreError> {
    let event_id = EXECUTION_EVENT_ID
        .with(Cell::get)
        .unwrap_or_else(Uuid::new_v4);
    let (operation_id, correlation_id) = correlation_ids(&request.body, event_id);
    let outcome = command_outcome(response);
    let error_class = response
        .error
        .as_ref()
        .map(|error| bounded(&error.code, 64));
    let mut attributes = BTreeMap::new();
    attributes.insert(
        "command".into(),
        Value::String(bounded(&request.command, 96)),
    );
    EventEnvelope::with_event_id(
        event_id,
        if response.ok {
            Severity::Info
        } else {
            Severity::Warn
        },
        Component::Daemon,
        EventKind::CommandCompleted,
        Some(request.request_id.as_str().to_string()),
        Some(operation_id),
        Some(correlation_id),
        actor_kind(request.actor.kind),
        bounded(&request.actor.name, 96),
        surface(request.actor.kind),
        outcome,
        error_class,
        attributes,
        "daemon-local",
    )
    .map_err(StoreError::invalid_state)
}

pub fn insert_event<C: GenericClient>(
    client: &mut C,
    event: &EventEnvelope,
) -> Result<(), StoreError> {
    let attributes = serde_json::to_value(&event.attributes)
        .map_err(|error| StoreError::invalid_state(error.to_string()))?;
    client.execute(
        "insert into observability_events
         (event_id,occurred_at,severity,component,event_kind,request_id,operation_id,correlation_id,
          actor_kind,actor_name,surface,outcome,error_class,attributes,source)
         values ($1,$2::text::timestamptz,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
         on conflict (event_id) do nothing",
        &[
            &event.event_id,
            &event.timestamp,
            &enum_name(&event.severity)?,
            &enum_name(&event.component)?,
            &enum_name(&event.event_kind)?,
            &event.request_id,
            &event.operation_id,
            &event.correlation_id,
            &event.actor_kind,
            &event.actor_name,
            &enum_name(&event.surface)?,
            &enum_name(&event.outcome)?,
            &event.error_class,
            &attributes,
            &event.source,
        ],
    )?;
    Ok(())
}

fn upsert_operation<C: GenericClient>(
    client: &mut C,
    event: &EventEnvelope,
) -> Result<(), StoreError> {
    client.execute(
        "insert into observability_operations
         (operation_id,request_id,correlation_id,command,actor_kind,actor_name,surface,outcome,error_class)
         values ($1,$2,$3,$4,$5,$6,$7,$8,$9)
         on conflict (operation_id) do update set request_id=excluded.request_id,
         command=excluded.command,actor_kind=excluded.actor_kind,actor_name=excluded.actor_name,
         surface=excluded.surface,outcome=excluded.outcome,
         error_class=excluded.error_class,completed_at=now()",
        &[&event.operation_id, &event.request_id, &event.correlation_id, &event_command(event),
          &event.actor_kind, &event.actor_name, &enum_name(&event.surface)?,
          &enum_name(&event.outcome)?, &event.error_class],
    )?;
    Ok(())
}

fn command_outcome(response: &CommandResponse) -> Outcome {
    if response.ok {
        Outcome::Succeeded
    } else if response
        .error
        .as_ref()
        .is_some_and(|error| error.code.starts_with("auth."))
    {
        Outcome::Denied
    } else if response
        .error
        .as_ref()
        .is_some_and(|error| error.code.contains("deadline"))
    {
        Outcome::Cancelled
    } else {
        Outcome::Failed
    }
}
fn actor_kind(kind: ActorKind) -> &'static str {
    match kind {
        ActorKind::Cli => "cli",
        ActorKind::VelocityPlugin => "velocity-plugin",
        ActorKind::PaperPlugin => "paper-plugin",
        ActorKind::Daemon => "daemon",
        ActorKind::Installer => "installer",
        ActorKind::WebOperator => "web-operator",
        ActorKind::Discord => "discord",
    }
}
fn surface(kind: ActorKind) -> Surface {
    match kind {
        ActorKind::Cli => Surface::Cli,
        ActorKind::WebOperator => Surface::Web,
        ActorKind::Discord => Surface::Discord,
        ActorKind::PaperPlugin => Surface::Paper,
        ActorKind::VelocityPlugin => Surface::Velocity,
        _ => Surface::Internal,
    }
}
fn enum_name<T: serde::Serialize>(value: &T) -> Result<String, StoreError> {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .ok_or_else(|| StoreError::invalid_state("observability enum serialization"))
}
fn bounded(value: &str, maximum: usize) -> String {
    value.chars().take(maximum).collect()
}
fn event_command(event: &EventEnvelope) -> String {
    event
        .attributes
        .get("command")
        .and_then(Value::as_str)
        .unwrap_or("[redacted]")
        .to_string()
}
