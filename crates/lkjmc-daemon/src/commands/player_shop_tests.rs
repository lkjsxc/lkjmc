use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use serde_json::{json, Value};

use super::player_shop::purchase_response;

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
    Ok(())
}
