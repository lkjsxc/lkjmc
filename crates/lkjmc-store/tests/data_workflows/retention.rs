use lkjmc_store::{data_workflows as workflows, instance, migrate};
use serde_json::json;
use uuid::Uuid;

use super::helpers::database;

#[test]
fn change_feed_retention_archives_then_expires() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut db) = database()? else {
        return Ok(());
    };
    let client = db.client_mut();
    migrate::apply(client)?;
    instance::insert(
        client,
        "retention-test",
        None,
        "folia",
        "stopped",
        &json!({}),
    )?;
    let id = Uuid::new_v4();
    workflows::create_runtime_intent(
        client,
        workflows::NewRuntimeIntent {
            id,
            instance_id: "retention-test",
            effect_kind: "start",
            requested_state: json!({"state":"running"}),
            fence: 1,
            correlation_id: Uuid::new_v4(),
        },
    )?;
    let revision = workflows::changes_after(client, 0, 10)?[0].feed_revision;
    client.execute(
        "update workflow_change_feed set created_at = now() - interval '31 days'
         where feed_revision = $1",
        &[&revision],
    )?;
    let archived = workflows::run_retention(client)?;
    assert_eq!(archived.archived, 1);
    assert_eq!(archived.deleted_active, 1);
    assert_eq!(archived.deleted_archive, 0);
    assert!(workflows::changes_after(client, 0, 10)?.is_empty());
    assert_eq!(workflows::retained_floor(client)?, Some(revision));

    client.execute(
        "update workflow_change_archive set created_at = now() - interval '366 days'
         where feed_revision = $1",
        &[&revision],
    )?;
    let expired = workflows::run_retention(client)?;
    assert_eq!(expired.deleted_archive, 1);
    assert_eq!(workflows::retained_floor(client)?, None);
    Ok(())
}
