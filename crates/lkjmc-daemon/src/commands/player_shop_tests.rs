use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use lkjmc_store::shop::{canonical_adventure_metadata, ShopItem};
use serde_json::{json, Value};

use crate::app::AppState;

use super::player_shop::{purchase, purchase_response};
use super::player_shop_delivery::{adventure_request, is_adventure_delivery};

#[test]
fn replay_response_hides_the_stored_delivery() -> Result<(), String> {
    let response = purchase_response(
        request(json!({"correlationId": id(1)}))?,
        lkjmc_store::shop::Purchase {
            item: ShopItem {
                id: "safe-item".to_string(),
                title_key: "shop.safe-item".to_string(),
                price_points: 10,
                metadata: json!({}),
            },
            duplicate: true,
            refundable: false,
        },
    );
    let body = response.body.ok_or("response body missing")?;
    assert_eq!(body["correlationId"], id(1));
    assert_eq!(body["duplicate"], true);
    assert_eq!(body["delivery"], Value::Null);
    assert_eq!(body["deliveryStatus"], "settled-replay");
    Ok(())
}

#[test]
fn adventure_metadata_never_falls_back_or_accepts_retired_metadata() -> Result<(), String> {
    let request = request(json!({}))?;
    assert!(is_adventure_delivery(&adventure_item(
        canonical_adventure_metadata()
    )));
    let compact = adventure_item(json!({"delivery":{"executor":"adventure-end-expedition"}}));
    assert!(!is_adventure_delivery(&compact));
    let error = adventure_request(&request, "player", &compact)
        .err()
        .ok_or("retired metadata accepted")?;
    assert_eq!(
        error.error.ok_or("missing error")?.code,
        "shop.unsupported_delivery"
    );
    let nested = adventure_request(
        &request,
        "player",
        &adventure_item(canonical_adventure_metadata()),
    )
    .map_err(|error| error.error.map(|value| value.code).unwrap_or_default())?;
    assert_eq!(nested.body["adventureId"], "end-expedition");
    Ok(())
}

#[test]
fn canonical_purchase_has_no_request_scoped_legal_preflight() -> Result<(), String> {
    assert_database_guard(purchase(&no_database_state(), purchase_request()?));
    Ok(())
}

#[test]
fn caller_delivery_cannot_trigger_adventure_preflight() -> Result<(), String> {
    let mut request = purchase_request()?;
    request.body["itemId"] = json!("custom-item");
    request.body["delivery"] = canonical_adventure_metadata()["delivery"].clone();
    assert_database_guard(purchase(&no_database_state(), request));
    assert_database_guard(purchase(&no_database_state(), purchase_request()?));
    Ok(())
}

fn purchase_request() -> Result<CommandEnvelope, String> {
    let body = json!({"playerUuid": id(1), "name": "shop-test",
        "itemId": "adventure-end-expedition", "correlationId": id(2)});
    request(body)
}

fn request(body: Value) -> Result<CommandEnvelope, String> {
    Ok(CommandEnvelope {
        request_id: CommandId::parse("shop request", "player.shop.purchase")
            .map_err(|error| error.to_string())?,
        actor: Actor {
            kind: ActorKind::PaperPlugin,
            name: "untrusted-body".to_string(),
        },
        command: "player.shop.purchase".to_string(),
        body,
    })
}

fn adventure_item(metadata: Value) -> ShopItem {
    ShopItem {
        id: "adventure-end-expedition".to_string(),
        title_key: "shop.item.adventure-end-expedition".to_string(),
        price_points: 250,
        metadata,
    }
}

fn id(value: u8) -> String {
    format!("00000000-0000-0000-0000-{value:012}")
}

fn assert_database_guard(response: lkjmc_core::command::CommandResponse) {
    assert_eq!(
        response.error.map(|error| error.code).as_deref(),
        Some("database.not_configured")
    );
}

fn no_database_state() -> AppState {
    AppState::with_config_path(
        None,
        1,
        "/tmp/config".to_string(),
        "/tmp/logs".to_string(),
        "/tmp/jars".to_string(),
        "/tmp/data".to_string(),
        None,
        None,
        None,
    )
}
