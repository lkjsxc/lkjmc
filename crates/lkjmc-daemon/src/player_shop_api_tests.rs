use serde_json::json;

use crate::player_shop_api::supported_delivery;

#[test]
fn supports_adventure_delivery_without_minecraft_item_material() {
    assert!(supported_delivery(
        &json!({"delivery":{"executor":"adventure","adventureId":"resource-rush"}})
    ));
    assert!(supported_delivery(
        &json!({"delivery":{"executor":"adventure-end-expedition"}})
    ));
    assert!(!supported_delivery(
        &json!({"delivery":{"executor":"unknown"}})
    ));
}
