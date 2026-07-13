use lkjmc_store::{instance, migrate};
use serde_json::json;
use uuid::Uuid;

use super::{helpers::database, support};

#[test]
fn schema_cutover_pass() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut db) = database()? else {
        return Ok(());
    };
    let client = db.client_mut();
    client.batch_execute("create table schema_migrations(version integer primary key, name text not null, checksum text, applied_at timestamptz default now())")?;
    for migration in migrate::migrations().into_iter().take(44) {
        client.batch_execute(migration.sql)?;
    }
    let player_id = Uuid::new_v4();
    client.execute(
        "insert into player_identities(player_uuid,current_name) values($1,'Legacy')",
        &[&player_id],
    )?;
    client.execute("insert into player_profile_snapshots(id,player_uuid,scope,revision,payload_format,payload,sha256,source_instance,metadata)
        values($1,$2,'profile',1,'java',decode('aced','hex'),$3,'old','{}')",
        &[&Uuid::new_v4(), &player_id, &support::TEST_SHA])?;
    instance::insert(
        client,
        "legacy-adventure",
        None,
        "folia",
        "running",
        &json!({}),
    )?;
    let adventure_id = Uuid::new_v4();
    client.execute("insert into temporary_instances(instance_id,owner_kind,owner_id,visibility,
        world_path,server_port,max_lifetime_seconds,retention_seconds,cleanup_policy,lifecycle_state,
        start_deadline_at,stop_deadline_at,expires_at,retain_until)
        values('legacy-adventure','adventure',$1,'hidden','/tmp/legacy-adventure',25567,60,0,
        'delete','ready',now(),now(),now(),now())", &[&adventure_id.to_string()])?;
    client.execute(
        "insert into adventure_sessions
        (id,adventure_kind,buyer_uuid,buyer_name,temporary_instance_id,points_cost,state,
         start_deadline_at,stop_deadline_at)
        values($1,'end-expedition',$2,'Legacy','legacy-adventure',0,'active',now(),now())",
        &[&adventure_id, &player_id],
    )?;
    client.batch_execute(migrate::migrations().last().unwrap().sql)?;
    assert_eq!(
        client
            .query_one(
                "select count(*) from player_profile_snapshot_quarantine",
                &[]
            )?
            .get::<_, i64>(0),
        1
    );
    assert_eq!(
        client
            .query_one("select count(*) from player_profile_snapshots", &[])?
            .get::<_, i64>(0),
        0
    );
    assert!(client
        .query_opt(
            "select to_regclass('temporary_transfer_intents')::text",
            &[]
        )?
        .unwrap()
        .get::<_, Option<String>>(0)
        .is_none());
    assert_eq!(
        client
            .query_one(
                "select state from adventure_sessions where id = $1",
                &[&adventure_id]
            )?
            .get::<_, String>(0),
        "start_observed"
    );
    assert_eq!(
        client
            .query_one(
                "select count(*) from workflow_change_feed where aggregate_id = $1",
                &[&adventure_id]
            )?
            .get::<_, i64>(0),
        1
    );
    Ok(())
}
