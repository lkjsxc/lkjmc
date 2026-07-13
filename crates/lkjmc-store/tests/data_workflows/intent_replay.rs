use lkjmc_store::{data_workflows as workflows, instance, migrate, player, shop};
use serde_json::json;
use uuid::Uuid;

use super::helpers::database;

#[test]
fn delivery_and_runtime_replays_bind_identity() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut db) = database()? else {
        return Ok(());
    };
    let client = db.client_mut();
    migrate::apply(client)?;
    let player_id = Uuid::new_v4();
    player::insert_identity(client, player_id, "Buyer")?;
    shop::upsert_item_with_metadata(
        client,
        "test",
        "shop.test",
        1,
        json!({"delivery":{"executor":"minecraft-item","material":"STONE","amount":1}}),
    )?;
    let purchase_id = Uuid::new_v4();
    client.execute(
        "insert into shop_purchases
         (id,player_uuid,item_id,price_points,correlation_id,metadata)
         values($1,$2,'test',1,$3,'{}')",
        &[&purchase_id, &player_id, &Uuid::new_v4()],
    )?;
    let delivery_id = Uuid::new_v4();
    let delivery_correlation = Uuid::new_v4();
    let delivery = || workflows::NewDelivery {
        id: delivery_id,
        purchase_id,
        player_uuid: player_id,
        delivery: json!({"executor":"minecraft-item","material":"STONE","amount":1}),
        correlation_id: delivery_correlation,
    };
    workflows::create_delivery(client, delivery())?;
    assert!(workflows::create_delivery(client, delivery())?.replay);
    let changed = workflows::NewDelivery {
        id: Uuid::new_v4(),
        ..delivery()
    };
    assert!(workflows::create_delivery(client, changed).is_err());

    instance::insert(client, "runtime-test", None, "folia", "stopped", &json!({}))?;
    let runtime_id = Uuid::new_v4();
    let runtime_correlation = Uuid::new_v4();
    let runtime = |id| workflows::NewRuntimeIntent {
        id,
        instance_id: "runtime-test",
        effect_kind: "start",
        requested_state: json!({"state":"running"}),
        fence: 1,
        correlation_id: runtime_correlation,
    };
    workflows::create_runtime_intent(client, runtime(runtime_id))?;
    assert!(workflows::create_runtime_intent(client, runtime(runtime_id))?.replay);
    assert!(workflows::create_runtime_intent(client, runtime(Uuid::new_v4())).is_err());
    Ok(())
}

#[test]
fn trusted_success_transitions_are_not_exposed() {
    let source = include_str!("../../src/data_workflows.rs");
    assert!(!source.contains("pub fn acknowledge"));
    assert!(!source.contains("pub fn observe"));
    assert!(!source.contains("pub fn succeed"));
}
