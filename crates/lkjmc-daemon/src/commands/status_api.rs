use std::time::{SystemTime, UNIX_EPOCH};

use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::{json, Value};

use crate::app::AppState;
use crate::dispatch as api;

pub fn status(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    status_response(request, status_body(state, None))
}

fn status_response(
    request: CommandEnvelope,
    result: Result<Value, lkjmc_store::error::StoreError>,
) -> CommandResponse {
    match result {
        Ok(body) => api::ok(request, body),
        Err(error) => api::database_error(request, error),
    }
}

pub(crate) fn status_body(
    state: &AppState,
    budget: Option<std::time::Duration>,
) -> Result<Value, lkjmc_store::error::StoreError> {
    let (database, counts, instances, instances_truncated) = database_status(state, budget)?;
    Ok(json!({
        "daemon": "running",
        "health": {"live": true, "readinessEndpoint": "/health/ready", "source": "daemon-local"},
        "startedAtUnixSeconds": unix_seconds(state.started_at()),
        "uptimeSeconds": uptime_seconds(state.started_at()),
        "database": database,
        "counts": counts,
        "instances": instances,
        "instanceSnapshot": {
            "source": "postgresql-latest-observation",
            "runtimeRefresh": false,
            "limit": lkjmc_store::status::INSTANCE_SNAPSHOT_LIMIT,
            "truncated": instances_truncated,
            "fieldCharacterLimits": {
                "id": lkjmc_store::status::INSTANCE_ID_CHAR_LIMIT,
                "observationMessage": lkjmc_store::status::OBSERVATION_MESSAGE_CHAR_LIMIT,
                "connectHost": lkjmc_store::status::CONNECT_HOST_CHAR_LIMIT,
                "proxyFailureReason": lkjmc_store::status::PROXY_FAILURE_CHAR_LIMIT
            }
        },
        "roots": {
            "config": state.config_root(),
            "data": state.data_root(),
            "log": state.log_root(),
            "jar": state.jar_root()
        },
        "socket": {"path": state.socket_path()},
        "http": match state.http_listener() {
            Some(address) => json!({"enabled": true, "address": address}),
            None => json!({"enabled": false})
        },
        "runtime": runtime_status(state),
        "commandLifecycle": {
            "admissionLimit": crate::command_lifecycle::ADMISSION_LIMIT,
            "deadlineSeconds": crate::command_lifecycle::DEADLINE.as_secs(),
            "queue": "none",
            "externalEffects": "denied-unproved"
        },
        "reconciler": {"enabled": state.reconciler_enabled()},
        "syncMaintenance": maintenance_status(state)
    }))
}

fn maintenance_status(state: &AppState) -> Value {
    let value = state.maintenance_diagnostics();
    json!({
        "running": value.running,
        "singletonCount": value.singleton_count,
        "completedRuns": value.completed_runs,
        "lastSuccessfulRun": value.last_successful_run,
        "archivedRows": value.archived_rows,
        "deletedRows": value.deleted_rows,
        "lastError": value.last_error
    })
}

fn runtime_status(state: &AppState) -> Value {
    let capabilities = state.runtime_capabilities();
    json!({
        "adapter": state.runtime_adapter_name(),
        "coordination": "per-instance-fenced",
        "capabilities": {
            "configuration": capabilities.configuration,
            "logs": capabilities.logs,
            "processIdentity": capabilities.process_identity,
            "readiness": capabilities.readiness,
            "recovery": capabilities.recovery,
            "secrets": capabilities.secrets,
            "storage": capabilities.storage
        }
    })
}

fn database_status(
    state: &AppState,
    budget: Option<std::time::Duration>,
) -> Result<(Value, Value, Value, bool), lkjmc_store::error::StoreError> {
    let empty_counts = json!({"instances": null, "activeSessions": null, "jarAssets": null, "presenceRecords": null});
    if state.database_url().is_none() {
        return Ok((
            json!({"configured": false, "connected": null, "poolSize": null}),
            empty_counts,
            Value::Null,
            false,
        ));
    }
    let mut client = match budget {
        Some(value) => state.request_database_connection_with_budget(value)?,
        None => state.request_database_connection()?,
    };
    let snapshot = lkjmc_store::status::snapshot(&mut client)?;
    let instances = snapshot
        .instances
        .into_iter()
        .map(instance_status)
        .collect::<Vec<_>>();
    let counts = snapshot.counts;
    Ok((
        json!({"configured": true, "connected": true, "poolSize": state.database_pool_size()}),
        json!({
            "instances": counts.instances,
            "activeSessions": counts.active_sessions,
            "jarAssets": counts.jar_assets,
            "presenceRecords": counts.presence_records
        }),
        json!(instances),
        snapshot.instances_truncated,
    ))
}

fn instance_status(row: lkjmc_store::status::InstanceStatus) -> Value {
    let connect_host = row
        .registered_host
        .clone()
        .unwrap_or_else(|| row.configured_host.clone());
    let connect_port = row.registered_port.map(i64::from).or(row.configured_port);
    let availability = crate::commands::instance_availability::evaluate(
        crate::commands::instance_availability::Input {
            kind: &row.kind,
            desired_state: &row.desired_state,
            process_healthy: row.process_healthy,
            connect_port,
            heartbeat_ready: row.heartbeat_ready,
            heartbeat_age_seconds: row.heartbeat_age_seconds,
            proxy_registration_desired: true,
            proxy_registered: row.proxy_registered,
            proxy_failure_reason: row.proxy_failure_reason.as_deref(),
            proxy_registration_age_seconds: row.proxy_registration_age_seconds,
        },
    );
    json!({
        "id": row.id,
        "idTruncated": row.id_truncated,
        "kind": row.kind,
        "desiredState": row.desired_state,
        "observedState": row.observed_state,
        "processHealthy": row.process_healthy,
        "ready": availability.ready,
        "readinessSource": availability.readiness_source,
        "readinessAgeSeconds": row.heartbeat_age_seconds,
        "observationAgeSeconds": row.observation_age_seconds,
        "observationMessage": row.observation_message,
        "pid": row.pid,
        "connectHost": connect_host,
        "connectPort": connect_port,
        "proxyRegistered": availability.proxy_registered,
        "proxyRegistrationAgeSeconds": row.proxy_registration_age_seconds,
        "joinable": availability.joinable,
        "joinDisabledReason": availability.join_disabled_reason,
        "diagnosticsTruncated": row.diagnostics_truncated
    })
}

fn unix_seconds(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

fn uptime_seconds(started_at: SystemTime) -> u64 {
    SystemTime::now()
        .duration_since(started_at)
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests;
