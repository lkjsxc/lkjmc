use lkjmc_store::{audit, jar};
use uuid::Uuid;

pub const TEST_SHA: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

pub fn reset_public_schema(
    client: &mut postgres::Client,
) -> Result<(), lkjmc_store::error::StoreError> {
    client.batch_execute(
        "select pg_advisory_lock(752647); drop schema public cascade; create schema public",
    )?;
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
