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
    let actor_kind = actor_kind(request)?;
    match execute_transaction(client, request, &actor_kind, mutation) {
        Ok(execution) => Ok(execution),
        Err(error) => journal::recover(client, request, &actor_kind, error),
    }
}

fn execute_transaction<F>(
    client: &mut Client,
    request: &CommandEnvelope,
    actor_kind: &str,
    mutation: F,
) -> Result<Execution, StoreError>
where
    F: FnOnce(&mut Transaction<'_>) -> Result<Value, StoreError>,
{
    let mut transaction = client.transaction()?;
    lock_request(&mut transaction, request)?;
    if !journal::insert_requested(&mut transaction, request, actor_kind)? {
        let replay = journal::replay(&mut transaction, request, actor_kind)?;
        transaction.commit()?;
        return Ok(replay);
    }
    let body = mutation(&mut transaction)?;
    let response = response::success(request, body);
    journal::store_terminal(&mut transaction, request, "succeeded", &response)?;
    transaction.commit()?;
    Ok(Execution::Outcome(response))
}

pub(super) fn lock_request(
    transaction: &mut Transaction<'_>,
    request: &CommandEnvelope,
) -> Result<(), StoreError> {
    transaction.query_one(
        "select pg_advisory_xact_lock(hashtextextended($1, 0))",
        &[&request.request_id.as_str()],
    )?;
    Ok(())
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
