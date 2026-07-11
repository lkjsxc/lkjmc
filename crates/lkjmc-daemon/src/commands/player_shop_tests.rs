use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use lkjmc_store::shop::ShopItem;
use serde_json::{json, Value};

use super::player_shop::purchase_response;
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

fn rejected(
    request: CommandEnvelope,
    item: ShopItem,
) -> Result<lkjmc_core::command::CommandResponse, String> {
    match adventure_request(&request, "direct-player", &item) {
        Ok(_) => Err("direct request unexpectedly prepared an adventure purchase".to_string()),
        Err(response) => Ok(response),
    }
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

fn response_error(response: lkjmc_core::command::CommandResponse) -> String {
    assert_confirmation(response)
        .map_or_else(|error| error, |_| "confirmation required".to_string())
}
