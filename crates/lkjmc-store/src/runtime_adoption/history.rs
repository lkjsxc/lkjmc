use postgres::GenericClient;
use uuid::Uuid;

use crate::error::StoreError;

use super::RuntimeOperation;

pub(super) fn append(
    client: &mut impl GenericClient,
    operation: &RuntimeOperation,
    phase: &str,
    outcome: &str,
    detail: Option<&str>,
) -> Result<(), StoreError> {
    client.execute(
        "insert into runtime_reconcile_history
         (instance_id,operation_id,correlation_id,fence,attempt,phase,outcome,detail)
         values ($1,$2,$3,$4,
           (select coalesce(max(attempt),0)+1 from runtime_reconcile_history where operation_id=$2),
           $5,$6,$7)",
        &[
            &operation.instance_id,
            &operation.id,
            &operation.correlation_id,
            &operation.fence,
            &phase,
            &outcome,
            &detail,
        ],
    )?;
    Ok(())
}

pub(super) fn from_row(row: &postgres::Row, replay: bool) -> RuntimeOperation {
    RuntimeOperation {
        id: row.get(0),
        instance_id: row.get(1),
        correlation_id: row.get(2),
        fence: row.get(3),
        intent: row.get(4),
        replay,
    }
}

pub fn count(client: &mut postgres::Client, operation_id: Uuid) -> Result<i64, StoreError> {
    Ok(client
        .query_one(
            "select count(*) from runtime_reconcile_history where operation_id=$1",
            &[&operation_id],
        )?
        .get(0))
}
