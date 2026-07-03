use lkjmc_store::{audit, jar};
use uuid::Uuid;

pub const TEST_SHA: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

pub fn prepare_isolated_schema(
    client: &mut postgres::Client,
) -> Result<String, lkjmc_store::error::StoreError> {
    let schema = format!("lkjmc_test_{}", Uuid::new_v4().simple());
    client.batch_execute(&format!(
        "create schema {schema}; set search_path to {schema}, public"
    ))?;
    Ok(schema)
}

pub fn drop_isolated_schema(
    client: &mut postgres::Client,
    schema: &str,
) -> Result<(), lkjmc_store::error::StoreError> {
    client.batch_execute(&format!("drop schema if exists {schema} cascade"))?;
    Ok(())
}

pub fn new_jar(id: Uuid) -> jar::NewJarAsset<'static> {
    jar::NewJarAsset {
        id,
        kind: "paper",
        project: "paper",
        channel: "stable",
        name: "paper-test.jar",
        path: "/opt/lkjmc/jars/papermc/paper/paper-test.jar",
        sha256: TEST_SHA,
        size_bytes: 3,
        source: "test",
    }
}

pub fn new_audit(id: Uuid) -> audit::NewAuditEvent<'static> {
    audit::NewAuditEvent {
        id,
        actor_kind: "cli",
        actor_name: "test",
        action: "instance.create",
        target_kind: "instance",
        target_id: "hub",
        result: "succeeded",
    }
}
