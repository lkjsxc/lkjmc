use postgres::GenericClient;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::StoreError;

pub struct EventQuery<'a> {
    pub request_id: Option<&'a str>,
    pub operation_id: Option<Uuid>,
    pub correlation_id: Option<Uuid>,
    pub limit: i64,
}

pub fn query<C: GenericClient>(
    client: &mut C,
    query: EventQuery<'_>,
) -> Result<Vec<Value>, StoreError> {
    let limit = query.limit.clamp(1, 500);
    let rows = client.query(
        "select event_id,occurred_at::text,severity,component,event_kind,request_id,operation_id,
         correlation_id,actor_kind,actor_name,surface,outcome,error_class,attributes,source
         from observability_events where occurred_at >= now() - interval '14 days'
         and ($1::text is null or request_id=$1)
         and ($2::uuid is null or operation_id=$2)
         and ($3::uuid is null or correlation_id=$3)
         order by occurred_at desc,event_id desc limit $4",
        &[
            &query.request_id,
            &query.operation_id,
            &query.correlation_id,
            &limit,
        ],
    )?;
    Ok(rows.into_iter().map(row_value).collect())
}

pub fn retain<C: GenericClient>(client: &mut C) -> Result<u64, StoreError> {
    let aged = client.execute(
        "delete from observability_events where occurred_at < now() - interval '14 days'",
        &[],
    )?;
    let capped = client.execute(
        "delete from observability_events where event_id in
         (select event_id from observability_events order by occurred_at desc,event_id desc offset 50000)",
        &[],
    )?;
    Ok(aged + capped)
}

fn row_value(row: postgres::Row) -> Value {
    json!({
        "eventId": row.get::<_, Uuid>(0),
        "timestamp": row.get::<_, String>(1),
        "severity": row.get::<_, String>(2),
        "component": row.get::<_, String>(3),
        "eventKind": row.get::<_, String>(4),
        "requestId": row.get::<_, Option<String>>(5),
        "operationId": row.get::<_, Option<Uuid>>(6),
        "correlationId": row.get::<_, Option<Uuid>>(7),
        "actorKind": row.get::<_, String>(8),
        "actorName": row.get::<_, String>(9),
        "surface": row.get::<_, String>(10),
        "outcome": row.get::<_, String>(11),
        "errorClass": row.get::<_, Option<String>>(12),
        "attributes": row.get::<_, Value>(13),
        "source": row.get::<_, String>(14)
    })
}
