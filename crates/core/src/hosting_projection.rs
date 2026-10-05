//! Ownership-based Hosting allowances; shared-server visibility never changes quota.
use crate::{App, error::Result, system_message::SystemMessage};
use serde_json::{Value, json};
use sqlx::Row;
use uuid::Uuid;

pub async fn for_account(app: &App, account: Uuid) -> Result<Value> {
    // Keep the usage predicate identical to hosting::reserve_capacity: admitted
    // file guests reserve resources even though Minecraft is stopped.
    let row = sqlx::query(
        "SELECT r.server_count::bigint AS server_limit,r.concurrent_servers::bigint AS concurrent_limit,
                r.memory_mib::bigint AS memory_limit,r.cpu_millis::bigint AS cpu_limit,r.storage_mib AS storage_limit,
                (SELECT count(*) FROM servers WHERE owner=$1) AS owned_count,
                (SELECT coalesce(sum(storage_mib),0)::bigint FROM servers WHERE owner=$1) AS owned_storage,
                (SELECT count(*) FROM servers WHERE owner=$1 AND (desired='running' OR inspection IS NOT NULL)) AS reserved_count,
                (SELECT coalesce(sum(memory_mib),0)::bigint FROM servers WHERE owner=$1 AND (desired='running' OR inspection IS NOT NULL)) AS reserved_memory,
                (SELECT coalesce(sum(cpu_millis),0)::bigint FROM servers WHERE owner=$1 AND (desired='running' OR inspection IS NOT NULL)) AS reserved_cpu
         FROM accounts a JOIN trust_ranks r ON r.id=a.trust_rank WHERE a.id=$1",
    ).bind(account).fetch_one(&app.db).await?;
    let server_limit: i64 = row.get("server_limit");
    let concurrent_limit: i64 = row.get("concurrent_limit");
    let memory_limit: i64 = row.get("memory_limit");
    let cpu_limit: i64 = row.get("cpu_limit");
    let storage_limit: i64 = row.get("storage_limit");
    let owned_count: i64 = row.get("owned_count");
    let owned_storage: i64 = row.get("owned_storage");
    let reserved_count: i64 = row.get("reserved_count");
    let reserved_memory: i64 = row.get("reserved_memory");
    let reserved_cpu: i64 = row.get("reserved_cpu");
    let minimum_storage = if app.config.development {
        1024
    } else {
        crate::hosting_limits::MIN_SERVER_STORAGE_MIB
    };
    let remaining_storage = (storage_limit - owned_storage).max(0);
    let blocked = if owned_count >= server_limit {
        Some("text.hosting_server_limit_reached")
    } else if remaining_storage < minimum_storage {
        Some("text.hosting_storage_allowance_insufficient")
    } else if memory_limit < 512 || cpu_limit < 1000 {
        Some("text.hosting_resource_allowance_insufficient")
    } else {
        None
    };
    Ok(json!({
        "limits":{"server_count":server_limit,"concurrent_servers":concurrent_limit,"memory_mib":memory_limit,"cpu_millis":cpu_limit,"storage_mib":storage_limit},
        "owned":{"server_count":owned_count,"storage_mib":owned_storage},
        "reserved":{"server_count":reserved_count,"memory_mib":reserved_memory,"cpu_millis":reserved_cpu},
        "remaining":{"server_count":(server_limit-owned_count).max(0),"storage_mib":remaining_storage,"concurrent_servers":(concurrent_limit-reserved_count).max(0),"memory_mib":(memory_limit-reserved_memory).max(0),"cpu_millis":(cpu_limit-reserved_cpu).max(0)},
        "minimum_server_storage_mib":minimum_storage,
        "minimum_server_memory_mib":512,
        "minimum_server_cpu_millis":1000,
        "server_cpu_step_millis":1000,
        "can_create":blocked.is_none(),
        "creation_blocked_reason":blocked.map(SystemMessage::new)
    }))
}
