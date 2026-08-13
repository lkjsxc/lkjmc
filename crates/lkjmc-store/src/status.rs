use postgres::Client;

use crate::error::StoreError;

pub const INSTANCE_SNAPSHOT_LIMIT: usize = 32;
pub const INSTANCE_ID_CHAR_LIMIT: i32 = 128;
pub const OBSERVATION_MESSAGE_CHAR_LIMIT: i32 = 256;
pub const CONNECT_HOST_CHAR_LIMIT: i32 = 255;
pub const PROXY_FAILURE_CHAR_LIMIT: i32 = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceStatus {
    pub id: String,
    pub id_truncated: bool,
    pub kind: String,
    pub desired_state: String,
    pub observed_state: Option<String>,
    pub process_healthy: Option<bool>,
    pub pid: Option<i32>,
    pub observation_message: Option<String>,
    pub observation_age_seconds: Option<i64>,
    pub configured_host: String,
    pub configured_port: Option<i64>,
    pub heartbeat_ready: Option<bool>,
    pub heartbeat_age_seconds: Option<i64>,
    pub registered_host: Option<String>,
    pub registered_port: Option<i32>,
    pub proxy_registered: Option<bool>,
    pub proxy_failure_reason: Option<String>,
    pub proxy_registration_age_seconds: Option<i64>,
    pub diagnostics_truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusCounts {
    pub instances: i64,
    pub active_sessions: i64,
    pub jar_assets: i64,
    pub presence_records: i64,
}

pub struct StatusSnapshot {
    pub counts: StatusCounts,
    pub instances: Vec<InstanceStatus>,
    pub instances_truncated: bool,
}

pub fn snapshot(client: &mut Client) -> Result<StatusSnapshot, StoreError> {
    let selected_limit = i64::try_from(INSTANCE_SNAPSHOT_LIMIT + 1)
        .map_err(|_| StoreError::invalid_state("instance snapshot limit is invalid"))?;
    let rows = client.query(
        "with selected as (
           select row_number() over (order by i.id) as ordinal,
             left(i.id, $2) as id,
             char_length(i.id) > $2 as id_truncated,
             i.kind, i.desired_state, o.observed_state, o.healthy, o.pid,
             left(o.message, $3) as message,
             case when o.updated_at is null then null
                  else extract(epoch from now() - o.updated_at)::bigint end
                  as observation_age_seconds,
             case when jsonb_typeof(i.config->'connectHost') = 'string'
                  then left(coalesce(nullif(i.config->>'connectHost', ''), '127.0.0.1'), $4)
                  else '127.0.0.1' end as configured_host,
             case when coalesce(i.config->>'serverPort', '') ~ '^[0-9]{1,5}$'
                       and (i.config->>'serverPort')::integer between 1 and 65535
                  then (i.config->>'serverPort')::bigint else null end as configured_port,
             p.ready,
             extract(epoch from now() - p.last_heartbeat_at)::bigint as heartbeat_age_seconds,
             left(r.connect_host, $4) as registered_host,
             r.connect_port, r.registered,
             left(r.failure_reason, $5) as failure_reason,
             extract(epoch from now() - r.reported_at)::bigint
                  as proxy_registration_age_seconds,
             char_length(coalesce(o.message, '')) > $3
               or char_length(coalesce(i.config->>'connectHost', '')) > $4
               or char_length(coalesce(r.connect_host, '')) > $4
               or char_length(coalesce(r.failure_reason, '')) > $5 as diagnostics_truncated
           from instances i
           left join instance_observations o on o.instance_id = i.id
           left join instance_presence p on p.instance_id = i.id
           left join proxy_registrations r on r.instance_id = i.id
           order by i.id limit $1
         ), totals as (
           select
             (select count(*)::bigint from instances) as instances,
             (select count(*)::bigint from player_sessions where left_at is null) as active_sessions,
             (select count(*)::bigint from jar_assets) as jar_assets,
             (select count(*)::bigint from instance_presence) as presence_records
         )
         select t.instances, t.active_sessions, t.jar_assets, t.presence_records,
           s.id, s.id_truncated, s.kind, s.desired_state, s.observed_state, s.healthy,
           s.pid, s.message, s.observation_age_seconds, s.configured_host,
           s.configured_port, s.ready, s.heartbeat_age_seconds, s.registered_host,
           s.connect_port, s.registered, s.failure_reason, s.proxy_registration_age_seconds,
           s.diagnostics_truncated
         from totals t left join selected s on true
         order by s.ordinal nulls last",
        &[
            &selected_limit,
            &INSTANCE_ID_CHAR_LIMIT,
            &OBSERVATION_MESSAGE_CHAR_LIMIT,
            &CONNECT_HOST_CHAR_LIMIT,
            &PROXY_FAILURE_CHAR_LIMIT,
        ],
    )?;
    let first = rows
        .first()
        .ok_or_else(|| StoreError::invalid_state("status snapshot returned no totals row"))?;
    let counts = StatusCounts {
        instances: first.get(0),
        active_sessions: first.get(1),
        jar_assets: first.get(2),
        presence_records: first.get(3),
    };
    let mut instances = rows
        .into_iter()
        .filter_map(|row| {
            let id = row.get::<_, Option<String>>(4)?;
            Some(InstanceStatus {
                id,
                id_truncated: row.get(5),
                kind: row.get(6),
                desired_state: row.get(7),
                observed_state: row.get(8),
                process_healthy: row.get(9),
                pid: row.get(10),
                observation_message: row.get(11),
                observation_age_seconds: row.get(12),
                configured_host: row.get(13),
                configured_port: row.get(14),
                heartbeat_ready: row.get(15),
                heartbeat_age_seconds: row.get(16),
                registered_host: row.get(17),
                registered_port: row.get(18),
                proxy_registered: row.get(19),
                proxy_failure_reason: row.get(20),
                proxy_registration_age_seconds: row.get(21),
                diagnostics_truncated: row.get(22),
            })
        })
        .collect::<Vec<_>>();
    let instances_truncated = instances.len() > INSTANCE_SNAPSHOT_LIMIT;
    instances.truncate(INSTANCE_SNAPSHOT_LIMIT);
    Ok(StatusSnapshot {
        counts,
        instances,
        instances_truncated,
    })
}
