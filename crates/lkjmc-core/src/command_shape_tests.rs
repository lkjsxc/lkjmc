use serde_json::json;

use super::*;

#[test]
fn domain_shapes_reject_unbounded_members() {
    for (command, member, value) in [
        (
            "instance.create",
            "rcon",
            json!({"password":"secret","port":25575,"extra":true}),
        ),
        (
            "player.home.set",
            "location",
            json!({"world":"world","x":0,"y":64,"z":0,"extra":true}),
        ),
        (
            "player.teleport.request",
            "location",
            json!({"world":"world","x":0,"y":64,"z":0,"extra":true}),
        ),
        (
            "player.warp.set",
            "location",
            json!({"world":"world","x":0,"y":64,"z":0,"extra":true}),
        ),
        ("shop.item.upsert", "metadata", json!({"extra":true})),
        (
            "temporary.instance.create",
            "metadata",
            json!({"extra":true}),
        ),
    ] {
        let mut body = required_body(command);
        body.insert(member.to_string(), value);
        assert!(validate_body(command, &json!(body)).is_err(), "{command}");
    }
}

fn required_body(command: &str) -> serde_json::Map<String, serde_json::Value> {
    let Some(contract) = contract_for(command) else {
        return serde_json::Map::new();
    };
    let RequestContract::Fields(request) = &contract.request else {
        return serde_json::Map::new();
    };
    request
        .fields
        .iter()
        .filter(|(_, field)| field.required)
        .map(|(name, field)| (name.clone(), super::sample(&field.value_type)))
        .collect()
}
