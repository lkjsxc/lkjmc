use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use lkjmc_store::shop::ShopItem;
use serde_json::{json, Value};

use crate::app::AppState;

use super::player_shop::{purchase, purchase_response};
use super::player_shop_delivery::{adventure_request, is_adventure_delivery};

#[test]
fn replay_response_hides_the_stored_delivery() -> Result<(), String> {
    let request = CommandEnvelope {
        request_id: CommandId::parse("shop replay", "test").map_err(|error| error.to_string())?,
        actor: Actor {
            kind: ActorKind::Cli,
            name: "test".to_string(),
        },
        command: "player.shop.purchase".to_string(),
        body: json!({"correlationId": "00000000-0000-0000-0000-000000000001"}),
    };
    let response = purchase_response(
        request,
        lkjmc_store::shop::Purchase {
            item: lkjmc_store::shop::ShopItem {
                id: "safe-item".to_string(),
                title_key: "shop.safe-item".to_string(),
                price_points: 10,
                metadata: json!({"delivery": {"executor": "minecraft-item"}}),
            },
            duplicate: true,
            refundable: false,
        },
    );
    let body = response
        .body
        .ok_or_else(|| "response body missing".to_string())?;
    assert_eq!(
        body["correlationId"],
        "00000000-0000-0000-0000-000000000001"
    );
    assert_eq!(body["duplicate"], json!(true));
    assert_eq!(body["refundable"], json!(false));
    assert_eq!(body["delivery"], Value::Null);
    assert_eq!(body["deliveryStatus"], "settled-replay");
    Ok(())
}

#[test]
fn shop_adventure_requires_caller_consent_without_synthesizing_it() -> Result<(), String> {
    for metadata in [adventure_delivery(), legacy_adventure_delivery()] {
        assert!(is_adventure_delivery(&metadata));
        for body in [json!({}), json!({"acceptMinecraftEula": false})] {
            assert_confirmation(rejected(request(body)?, adventure_item(metadata.clone()))?)?;
        }
    }
    let nested = adventure_request(
        &request(json!({"acceptMinecraftEula": true}))?,
        "menu-player",
        &adventure_item(adventure_delivery()),
    )
    .map_err(response_error)?;
    assert_eq!(nested.body["acceptMinecraftEula"], json!(true));
    assert_eq!(nested.body["playerName"], json!("menu-player"));
    Ok(())
}

#[test]
fn unconfirmed_public_adventure_purchase_skips_the_database_guard() -> Result<(), String> {
    for consent in [None, Some(false)] {
        let response = purchase(&no_database_state(), purchase_request(consent)?);
        assert_confirmation(response)?;
    }
    Ok(())
}

#[test]
fn confirmed_public_adventure_purchase_reaches_the_database_guard() -> Result<(), String> {
    let response = purchase(&no_database_state(), purchase_request(Some(true))?);
    assert_database_guard(response)?;
    let mut untrusted = purchase_request(None)?;
    untrusted.body["itemId"] = json!("ordinary-item");
    untrusted.body["delivery"] = adventure_delivery();
    assert_database_guard(purchase(&no_database_state(), untrusted))
}

fn rejected(
    request: CommandEnvelope,
    item: ShopItem,
) -> Result<lkjmc_core::command::CommandResponse, String> {
    match adventure_request(&request, "direct-player", &item) {
        Ok(_) => Err("direct request unexpectedly prepared an adventure purchase".to_string()),
        Err(response) => Ok(response),
    }
}

fn purchase_request(consent: Option<bool>) -> Result<CommandEnvelope, String> {
    let mut body = json!({
        "playerUuid": "00000000-0000-0000-0000-000000000001",
        "name": "shop-test", "itemId": "adventure-end-expedition",
        "correlationId": "00000000-0000-0000-0000-000000000002"
    });
    if let Some(consent) = consent {
        body["acceptMinecraftEula"] = json!(consent);
    }
    request(body)
}

fn request(body: Value) -> Result<CommandEnvelope, String> {
    Ok(CommandEnvelope {
        request_id: CommandId::parse("shop request", "player.shop.purchase")
            .map_err(|error| error.to_string())?,
        actor: Actor {
            kind: ActorKind::PaperPlugin,
            name: "shop-test".to_string(),
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

fn adventure_delivery() -> Value {
    json!({"delivery":{"executor":"adventure","adventureId":"end-expedition"}})
}

fn legacy_adventure_delivery() -> Value {
    json!({"delivery":{"executor":"adventure-end-expedition"}})
}

fn assert_confirmation(response: lkjmc_core::command::CommandResponse) -> Result<(), String> {
    if response.ok || response.body.is_some() {
        return Err("expected a bodyless confirmation-required response".to_string());
    }
    match response.error {
        Some(error) if error.code == "adventure.confirmation_required" && !error.retryable => {
            Ok(())
        }
        _ => Err("expected the shared confirmation-required response".to_string()),
    }
}

fn assert_database_guard(response: lkjmc_core::command::CommandResponse) -> Result<(), String> {
    match response.error {
        Some(error) if error.code == "database.not_configured" && !error.retryable => Ok(()),
        _ => Err("expected the no-database guard".to_string()),
    }
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

fn response_error(response: lkjmc_core::command::CommandResponse) -> String {
    assert_confirmation(response)
        .map_or_else(|error| error, |_| "confirmation required".to_string())
}
