use postgres::Client;
use serde_json::Value;

use super::{history, RuntimeOperation};
use crate::data_workflows::append;
use crate::error::StoreError;

pub fn observe(
    client: &mut Client,
    operation: &RuntimeOperation,
    observation: &Value,
    outcome: &str,
    detail: Option<&str>,
) -> Result<bool, StoreError> {
    if !observation.is_object() || !matches!(outcome, "succeeded" | "failed" | "unknown") {
        return Err(StoreError::invalid_state("invalid runtime observation"));
    }
    let mut tx = client.transaction()?;
    let owned = tx
        .query_opt(
            "select 1 from runtime_instance_fences
         where instance_id=$1 and fence=$2 and operation_id=$3 and correlation_id=$4 for update",
            &[
                &operation.instance_id,
                &operation.fence,
                &operation.id,
                &operation.correlation_id,
            ],
        )?
        .is_some();
    if !owned {
        history::append(&mut tx, operation, "observation", "stale", detail)?;
        tx.commit()?;
        return Ok(false);
    }
    let state = if outcome == "succeeded" {
        "observed"
    } else if outcome == "failed" {
        "failed"
    } else {
        "pending_observation"
    };
    let revision: i64 = tx
        .query_one(
            "update runtime_effect_workflows set state=$2, revision=revision+1,
         observation=$3, failure_reason=$4, updated_at=now()
         where operation_id=$1 returning revision",
            &[&operation.id, &state, observation, &detail],
        )?
        .get(0);
    let observed_state = observation
        .get("observedState")
        .and_then(Value::as_str)
        .unwrap_or("runtime-unknown");
    let healthy = observation
        .get("healthy")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let pid = observation
        .get("identity")
        .and_then(|value| value.get("pid"))
        .and_then(Value::as_i64)
        .and_then(|value| i32::try_from(value).ok());
    let message = observation
        .get("message")
        .and_then(Value::as_str)
        .or(detail);
    tx.execute(
        "insert into instance_observations
         (instance_id,observed_state,pid,healthy,started_at,message)
         values ($1,$2,$3,$4,case when $4 then now() else null end,$5)
         on conflict (instance_id) do update set
           observed_state=excluded.observed_state,pid=excluded.pid,healthy=excluded.healthy,
           started_at=case when excluded.healthy then coalesce(instance_observations.started_at,now())
                           else instance_observations.started_at end,
           message=excluded.message,updated_at=now()",
        &[&operation.instance_id, &observed_state, &pid, &healthy, &message],
    )?;
    history::append(&mut tx, operation, "outcome", outcome, detail)?;
    append(
        &mut tx,
        "runtime",
        operation.id,
        revision,
        operation.correlation_id,
        state,
    )?;
    tx.commit()?;
    Ok(true)
}
