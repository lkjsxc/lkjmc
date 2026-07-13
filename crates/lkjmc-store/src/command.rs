use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use postgres::{Client, Transaction};
use serde_json::Value;

use crate::error::StoreError;

pub enum Execution {
    Outcome(CommandResponse),
    Conflict,
}

pub struct JournalOutcome {
    pub result: String,
    pub response: CommandResponse,
}

pub fn execute_desired<F>(
    client: &mut Client,
    request: &CommandEnvelope,
    mutation: F,
) -> Result<Execution, StoreError>
where
    F: FnOnce(&mut Transaction<'_>) -> Result<Value, StoreError>,
{
    let id = request.request_id.as_str();
    client.query_one("select pg_advisory_lock(hashtextextended($1, 0))", &[&id])?;
    let result = execute_locked(client, request, mutation);
    let unlocked = client.query_one("select pg_advisory_unlock(hashtextextended($1, 0))", &[&id]);
    match (result, unlocked) {
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error.into()),
        (Ok(value), Ok(_)) => Ok(value),
    }
}

fn execute_locked<F>(
    client: &mut Client,
    request: &CommandEnvelope,
    mutation: F,
) -> Result<Execution, StoreError>
where
    F: FnOnce(&mut Transaction<'_>) -> Result<Value, StoreError>,
{
    let actor_kind = actor_kind(request)?;
    if !journal::insert_requested(client, request, &actor_kind)? {
        return journal::replay(client, request, &actor_kind);
    }
    match run_mutation(client, request, mutation) {
        Ok(response) => Ok(Execution::Outcome(response)),
        Err(error) => journal::recover(client, request, error),
    }
}

fn run_mutation<F>(
    client: &mut Client,
    request: &CommandEnvelope,
    mutation: F,
) -> Result<CommandResponse, StoreError>
where
    F: FnOnce(&mut Transaction<'_>) -> Result<Value, StoreError>,
{
    let mut transaction = client.transaction()?;
    let body = mutation(&mut transaction)?;
    let response = response::success(request, body);
    journal::store_terminal(&mut transaction, request, "succeeded", &response)?;
    transaction.commit()?;
    Ok(response)
}

pub fn lookup(client: &mut Client, id: &str) -> Result<Option<JournalOutcome>, StoreError> {
    journal::lookup(client, id)
}

fn actor_kind(request: &CommandEnvelope) -> Result<String, StoreError> {
    serde_json::to_value(request.actor.kind)
        .ok()
        .and_then(|value| value.as_str().map(ToString::to_string))
        .ok_or_else(|| StoreError::invalid_state("actor kind is not serializable"))
}

mod journal;
mod response;
