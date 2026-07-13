use lkjmc_store::{data_workflows as workflows, instance, migrate};
use serde_json::json;
use uuid::Uuid;

use super::helpers::database;

#[test]
fn change_feed_resume_is_explicit_across_retention_floors(
) -> Result<(), lkjmc_store::error::StoreError> {
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
    let first = runtime_intent(client)?;
    let first_revision = feed_revision(client, first)?;
    assert_eq!(workflows::retained_floor(client)?, Some(first_revision));
    assert_eq!(
        workflows::changes_after(client, first_revision - 1, 10)?,
        workflows::ResumeResult::ReloadRequired {
            active_floor: Some(first_revision)
        },
    );
    assert_eq!(
        workflows::changes_after(client, first_revision, 10)?,
        workflows::ResumeResult::Changes(Vec::new()),
    );

    client.execute(
        "update workflow_change_feed set created_at = now() - interval '31 days'
         where feed_revision = $1",
        &[&first_revision],
    )?;
    let second = runtime_intent(client)?;
    let second_revision = feed_revision(client, second)?;
    let archived = workflows::run_retention(client)?;
    assert_eq!((archived.archived, archived.deleted_active), (1, 1));
    assert_eq!(workflows::retained_floor(client)?, Some(second_revision));
    assert_eq!(
        workflows::changes_after(client, first_revision, 10)?,
        workflows::ResumeResult::ReloadRequired {
            active_floor: Some(second_revision)
        },
    );
    assert_eq!(
        workflows::changes_after(client, second_revision, 10)?,
        workflows::ResumeResult::Changes(Vec::new()),
    );

    client.execute(
        "update workflow_change_feed set created_at = now() - interval '31 days'",
        &[],
    )?;
    workflows::run_retention(client)?;
    assert_eq!(workflows::retained_floor(client)?, None);
    assert_eq!(
        workflows::changes_after(client, first_revision, 10)?,
        workflows::ResumeResult::ReloadRequired { active_floor: None },
    );
    assert_eq!(
        workflows::changes_after(client, second_revision, 10)?,
        workflows::ResumeResult::Changes(Vec::new()),
    );

    client.execute(
        "update workflow_change_archive set created_at = now() - interval '366 days'",
        &[],
    )?;
    workflows::run_retention(client)?;
    assert_eq!(
        workflows::changes_after(client, first_revision, 10)?,
        workflows::ResumeResult::ReloadRequired { active_floor: None },
    );
    Ok(())
}

fn runtime_intent(client: &mut postgres::Client) -> Result<Uuid, lkjmc_store::error::StoreError> {
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
    Ok(id)
}

fn feed_revision(
    client: &mut postgres::Client,
    id: Uuid,
) -> Result<i64, lkjmc_store::error::StoreError> {
    Ok(client
        .query_one(
            "select feed_revision from workflow_change_feed where aggregate_id = $1",
            &[&id],
        )?
        .get(0))
}
