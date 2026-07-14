use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use postgres::{Client, GenericClient, Transaction};
use serde_json::Value;

use crate::error::StoreError;

use super::{lock_request, response, Execution, JournalOutcome};

pub(super) fn insert_requested<C: GenericClient>(
    client: &mut C,
    request: &CommandEnvelope,
    actor_kind: &str,
) -> Result<bool, StoreError> {
    let inserted = client.execute(
        "insert into commands
         (id, actor_kind, actor_name, command, body, result, metadata)
         values ($1, $2, $3, $4, $5, 'requested', '{}'::jsonb)
         on conflict (id) do nothing",
        &[
            &request.request_id.as_str(),
            &actor_kind,
            &request.actor.name,
            &request.command,
            &request.body,
        ],
    )?;
    Ok(inserted == 1)
}

pub(super) fn replay<C: GenericClient>(
    client: &mut C,
    request: &CommandEnvelope,
    actor_kind: &str,
) -> Result<Execution, StoreError> {
    let row = client.query_one(
        "select actor_kind, actor_name, command, body, result, metadata
         from commands where id = $1",
        &[&request.request_id.as_str()],
    )?;
    let matches = row.get::<_, String>(0) == actor_kind
        && row.get::<_, String>(1) == request.actor.name
        && row.get::<_, String>(2) == request.command
        && row.get::<_, Value>(3) == request.body;
    if !matches {
        return Ok(Execution::Conflict);
    }
    let result = row.get::<_, String>(4);
    if result == "requested" {
        let response = response::failure(
            request,
            "request.interrupted",
            "previous request worker ended before a terminal outcome",
            false,
        );
        terminalize(client, request, "failed", &response)?;
        crate::observability::record_command(client, request, &response)?;
        return Ok(Execution::Outcome(response));
    }
    Ok(Execution::Outcome(response::from_metadata(row.get(5))?))
}

pub(super) fn recover(
    client: &mut Client,
    request: &CommandEnvelope,
    actor_kind: &str,
    error: StoreError,
) -> Result<Execution, StoreError> {
    match recover_transaction(client, request, actor_kind, &error) {
        Ok(execution) => Ok(execution),
        Err(_) => Err(error),
    }
}

fn recover_transaction(
    client: &mut Client,
    request: &CommandEnvelope,
    actor_kind: &str,
    error: &StoreError,
) -> Result<Execution, StoreError> {
    let mut transaction = client.transaction()?;
    lock_request(&mut transaction, request)?;
    if !insert_requested(&mut transaction, request, actor_kind)? {
        let replay = replay(&mut transaction, request, actor_kind)?;
        transaction.commit()?;
        return Ok(replay);
    }
    let deadline = error.is_deadline();
    let response = response::failure(
        request,
        if deadline {
            "command.deadline_exceeded"
        } else {
            "database.error"
        },
        error.to_string(),
        deadline,
    );
    store_terminal(
        &mut transaction,
        request,
        if deadline { "cancelled" } else { "failed" },
        &response,
    )?;
    crate::observability::record_command(&mut transaction, request, &response)?;
    transaction.commit()?;
    Ok(Execution::Outcome(response))
}

pub(super) fn lookup(client: &mut Client, id: &str) -> Result<Option<JournalOutcome>, StoreError> {
    let row = client.query_opt(
        "select result, metadata from commands where id = $1",
        &[&id],
    )?;
    row.map(|row| {
        Ok(JournalOutcome {
            result: row.get(0),
            response: response::from_metadata(row.get(1))?,
        })
    })
    .transpose()
}

pub(super) fn store_terminal(
    transaction: &mut Transaction<'_>,
    request: &CommandEnvelope,
    result: &str,
    response: &CommandResponse,
) -> Result<(), StoreError> {
    update_terminal(transaction, request, result, response)
}

fn terminalize<C: GenericClient>(
    client: &mut C,
    request: &CommandEnvelope,
    result: &str,
    response: &CommandResponse,
) -> Result<(), StoreError> {
    update_terminal(client, request, result, response)
}

fn update_terminal<C: GenericClient>(
    client: &mut C,
    request: &CommandEnvelope,
    result: &str,
    response: &CommandResponse,
) -> Result<(), StoreError> {
    let metadata = response::metadata(response)?;
    let updated = client.execute(
        "update commands set result = $2, metadata = $3, completed_at = now()
         where id = $1 and result = 'requested'",
        &[&request.request_id.as_str(), &result, &metadata],
    )?;
    if updated != 1 {
        return Err(StoreError::invalid_state(
            "command journal did not terminalize exactly one request",
        ));
    }
    Ok(())
}
