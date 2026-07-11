use serde_json::json;

use crate::commands::player_shop::supported_delivery;

#[test]
fn only_the_canonical_item_has_adventure_delivery() {
    let canonical = json!({"delivery":{"executor":"adventure","adventureId":"end-expedition"}});
    assert!(supported_delivery("adventure-end-expedition", &canonical));
    assert!(!supported_delivery("resource-rush", &canonical));
    assert!(!supported_delivery(
        "adventure-end-expedition",
        &json!({"delivery":{"executor":"adventure-end-expedition"}})
    ));
    assert!(!supported_delivery(
        "safe",
        &json!({"delivery":{"executor":"unknown"}})
    ));
}
