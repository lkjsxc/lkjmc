use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use postgres::{Client, GenericClient, Transaction};
use serde_json::Value;

use crate::error::StoreError;

use super::{response, Execution, JournalOutcome};

pub(super) fn insert_requested(
    client: &mut Client,
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

pub(super) fn replay(
    client: &mut Client,
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
        return Ok(Execution::Outcome(response));
    }
    Ok(Execution::Outcome(response::from_metadata(row.get(5))?))
}

pub(super) fn recover(
    client: &mut Client,
    request: &CommandEnvelope,
    error: StoreError,
) -> Result<Execution, StoreError> {
    let row = match client.query_opt(
        "select result, metadata from commands where id = $1",
        &[&request.request_id.as_str()],
    ) {
        Ok(Some(row)) => row,
        _ => return Err(error),
    };
    if row.get::<_, String>(0) != "requested" {
        return Ok(Execution::Outcome(response::from_metadata(row.get(1))?));
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
    terminalize(
        client,
        request,
        if deadline { "cancelled" } else { "failed" },
        &response,
    )?;
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

fn terminalize(
    client: &mut Client,
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
