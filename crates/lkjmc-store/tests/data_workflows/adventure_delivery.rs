use lkjmc_store::{data_workflows as workflows, instance, migrate, player, shop};
use serde_json::json;
use uuid::Uuid;

use super::helpers::{database, fail_feed};

#[test]
fn delivery_crash_matrix() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut db) = database()? else {
        return Ok(());
    };
    let client = db.client_mut();
    migrate::apply(client)?;
    let player_id = Uuid::new_v4();
    player::insert_identity(client, player_id, "Buyer")?;
    lkjmc_store::points::grant(client, player_id, 20, "test")?;
    shop::upsert_item_with_metadata(
        client,
        "stone",
        "shop.stone",
        5,
        json!({"delivery":{"executor":"minecraft-item","material":"STONE","amount":64}}),
    )?;
    let item = shop::get_item(client, "stone")?
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("shop item missing"))?;
    fail_feed(client, "delivery")?;
    assert!(shop::purchase(client, player_id, &item, Uuid::new_v4()).is_err());
    let balance = lkjmc_store::points::balance(client, player_id)?;
    assert_eq!(balance, 20);
    assert_eq!(
        client
            .query_one("select count(*) from shop_purchases", &[])?
            .get::<_, i64>(0),
        0
    );
    assert_eq!(
        client
            .query_one("select count(*) from item_delivery_workflows", &[])?
            .get::<_, i64>(0),
        0
    );
    Ok(())
}

#[test]
fn delivery_refund_fails_intent_atomically() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut db) = database()? else {
        return Ok(());
    };
    let client = db.client_mut();
    migrate::apply(client)?;
    let player_id = Uuid::new_v4();
    let correlation = Uuid::new_v4();
    player::insert_identity(client, player_id, "Buyer")?;
    lkjmc_store::points::grant(client, player_id, 20, "test")?;
    shop::upsert_item_with_metadata(
        client,
        "stone",
        "shop.stone",
        5,
        json!({"delivery":{"executor":"minecraft-item","material":"STONE","amount":64}}),
    )?;
    let item = shop::get_item(client, "stone")?
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("shop item missing"))?;
    shop::purchase(client, player_id, &item, correlation)?;
    assert!(shop::refund_purchase(
        client,
        player_id,
        correlation,
        "shop.refund"
    )?);
    let state: String = client
        .query_one(
            "select state from item_delivery_workflows where correlation_id = $1",
            &[&correlation],
        )?
        .get(0);
    assert_eq!(state, "failed");
    assert_eq!(lkjmc_store::points::balance(client, player_id)?, 20);
    Ok(())
}

#[test]
fn adventure_crash_matrix() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut db) = database()? else {
        return Ok(());
    };
    let client = db.client_mut();
    migrate::apply(client)?;
    let player_id = Uuid::new_v4();
    player::insert_identity(client, player_id, "Buyer")?;
    instance::insert(
        client,
        "adventure-test",
        None,
        "folia",
        "stopped",
        &json!({}),
    )?;
    client.execute("insert into temporary_instances(instance_id,owner_kind,owner_id,visibility,
        world_path,server_port,max_lifetime_seconds,retention_seconds,cleanup_policy,lifecycle_state,
        start_deadline_at,stop_deadline_at,expires_at,retain_until)
        values('adventure-test','adventure','owner','hidden','/tmp/adventure-test',25566,60,0,
        'delete','created',now(),now(),now(),now())", &[])?;
    fail_feed(client, "adventure")?;
    let mut tx = client.transaction()?;
    let result = lkjmc_store::temporary::insert_session(
        &mut tx,
        lkjmc_store::temporary::NewAdventureSession {
            id: Uuid::new_v4(),
            adventure_kind: "end-expedition",
            buyer_uuid: player_id,
            buyer_name: "Buyer",
            temporary_instance_id: "adventure-test",
            points_cost: 0,
            points_ledger_id: None,
            state: "pending",
            start_deadline_seconds: 30,
            stop_deadline_seconds: 60,
            metadata: json!({}),
        },
    );
    assert!(result.is_err());
    drop(tx);
    assert_eq!(
        client
            .query_one("select count(*) from adventure_sessions", &[])?
            .get::<_, i64>(0),
        0
    );
    client.batch_execute(
        "drop trigger workflow_failpoint on workflow_change_feed; drop function fail_feed()",
    )?;
    fail_feed(client, "runtime")?;
    assert!(workflows::create_runtime_intent(
        client,
        workflows::NewRuntimeIntent {
            id: Uuid::new_v4(),
            instance_id: "adventure-test",
            effect_kind: "start",
            requested_state: json!({"state":"running"}),
            fence: 1,
            correlation_id: Uuid::new_v4(),
        }
    )
    .is_err());
    assert_eq!(
        client
            .query_one("select count(*) from runtime_effect_workflows", &[])?
            .get::<_, i64>(0),
        0
    );
    Ok(())
}
