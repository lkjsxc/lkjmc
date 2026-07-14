#[allow(dead_code)]
mod support;

use lkjmc_store::{claims, migrate, sync};
use serde_json::json;
use uuid::Uuid;

#[test]
fn every_domain_is_revisioned_and_rollback_does_not_publish(
) -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    let player = Uuid::new_v4();
    let session = Uuid::new_v4();
    client.execute(
        "insert into player_identities(player_uuid,current_name,metadata) values($1,'Sync','{}')",
        &[&player],
    )?;
    client.execute(
        "insert into player_sessions(id,player_uuid,current_server,revision) values($1,$2,'hub',1)",
        &[&session, &player],
    )?;
    client.execute(
        "insert into player_settings(player_uuid,language) values($1,'en')",
        &[&player],
    )?;
    client.execute(
        "insert into player_profile_snapshots(id,player_uuid,scope,revision,session_id,
         session_revision,lease_fence,expected_snapshot_revision,correlation_id,schema_name,
         envelope,canonical_json,sha256,source_instance)
         values($1,$2,'profile',1,$3,1,1,0,$4,'lkjmc-profile-one',$5,$6,$7,'hub')",
        &[
            &Uuid::new_v4(),
            &player,
            &session,
            &Uuid::new_v4(),
            &json!({"schema":"lkjmc-profile-one"}),
            &&b"{}"[..],
            &"0".repeat(64),
        ],
    )?;
    client.execute(
        "insert into instances(id,kind,desired_state,config) values('hub','paper','running','{}')",
        &[],
    )?;
    client.execute(
        "insert into instance_presence(instance_id,last_heartbeat_at,ready,metadata)
         values('hub',now(),true,'{}')",
        &[],
    )?;
    let claim_id = Uuid::new_v4();
    claims::create_claim(
        client,
        claims::NewClaim {
            id: claim_id,
            owner_uuid: player,
            owner_name: "Sync",
            name: "Base",
            instance_id: "hub",
            world_name: "world",
            chunk_x: 1,
            chunk_z: 2,
        },
    )?;
    client.execute(
        "insert into admin_roles(id,title_key,permissions) values('sync-role','sync',
         '[\"lkjmc.admin.operator\"]')",
        &[],
    )?;
    client.execute(
        "insert into admin_grants(id,principal_kind,principal_id,role_id,reason,
         granted_by_kind,granted_by_id) values($1,'player',$2,'sync-role','test','cli','test')",
        &[&Uuid::new_v4(), &player.to_string()],
    )?;
    client.execute(
        "insert into shop_items(id,title_key,price_points) values('sync-item','sync.item',1)",
        &[],
    )?;

    let keys = [
        ("permissions", format!("player:{player}")),
        ("claims", "hub".into()),
        ("menus", "global".into()),
        ("profiles", format!("{player}:profile")),
        ("presence", "hub".into()),
        ("routing", "network".into()),
        ("settings", player.to_string()),
    ];
    for (domain, key) in keys {
        let sync::SnapshotResult::Available(snapshot) = sync::snapshot(client, domain, &key)?
        else {
            panic!("{domain}/{key} unavailable");
        };
        assert_eq!(snapshot.domain, domain);
        assert_eq!(snapshot.key, key);
        assert!(snapshot.revision > 0);
        assert!(snapshot.generated_at.ends_with('Z'));
        assert!(snapshot.payload.is_object());
    }
    let before = revision(client, "settings", &player.to_string())?;
    let mut tx = client.transaction()?;
    tx.execute(
        "update player_settings set language='ja' where player_uuid=$1",
        &[&player],
    )?;
    tx.rollback()?;
    assert_eq!(revision(client, "settings", &player.to_string())?, before);
    assert!(matches!(
        sync::changes_after(client, 0, 128)?,
        sync::FeedResult::Changes { .. }
    ));
    Ok(())
}

fn revision(
    client: &mut postgres::Client,
    domain: &str,
    key: &str,
) -> Result<i64, lkjmc_store::error::StoreError> {
    Ok(client
        .query_one(
            "select revision from sync_domain_revisions where domain=$1 and key=$2",
            &[&domain, &key],
        )?
        .get(0))
}
