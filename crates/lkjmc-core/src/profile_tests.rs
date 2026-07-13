use serde_json::{json, Value};

use crate::profile_validation::canonical_profile;

fn complete() -> Value {
    json!({
        "schema": "lkjmc-profile-one",
        "inventory": [{"slot": 2, "item": item("minecraft:stone")}],
        "armor": [],
        "offhand": null,
        "selectedHotbarSlot": 1,
        "enderChest": [],
        "experience": {"progress": 0.5, "level": 2, "total": 9},
        "vitals": {"health": 20.0, "food": 20, "saturation": 5.0, "air": 300},
        "potionEffects": [],
        "gameMode": "survival",
        "pluginData": [{"key": "lkjmc:rank", "value": "member"}],
        "homes": [{"name":"base","server":"hub","world":"minecraft:overworld",
            "x":1.0,"y":64.0,"z":2.0,"yaw":0.0,"pitch":0.0}],
        "warps": [],
        "points": 12,
        "achievements": ["lkjmc:first_login"],
        "settings": {"menuEnabled":true,"hudEnabled":true,"tipsEnabled":false,"privacy":"friends"},
        "language": "en-US"
    })
}

fn item(material: &str) -> Value {
    json!({"material":material,"amount":1,"damage":0,"customName":null,
        "lore":[],"enchantments":[],"customModelData":null})
}

#[test]
fn profile_format_safe_complete_and_canonical() {
    let input = serde_json::to_vec_pretty(&complete()).unwrap();
    let first = canonical_profile(&input).unwrap();
    let second = canonical_profile(&first.json).unwrap();
    assert_eq!(first.json, second.json);
    assert_eq!(first.sha256, second.sha256);
    assert!(!String::from_utf8(first.json).unwrap().contains(" \n"));
}

#[test]
fn profile_rejects_unknown_duplicate_unsafe_and_incomplete() {
    let encoded = serde_json::to_string(&complete()).unwrap();
    let duplicate = encoded.replacen(
        "\"schema\":\"lkjmc-profile-one\"",
        "\"schema\":\"lkjmc-profile-one\",\"schema\":\"other\"",
        1,
    );
    let error = canonical_profile(duplicate.as_bytes()).err().unwrap();
    assert!(error.contains("duplicate field `schema`"));
    let mut value = complete();
    value
        .as_object_mut()
        .unwrap()
        .insert("opaque".into(), json!("bytes"));
    assert!(canonical_profile(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut value = complete();
    value.as_object_mut().unwrap().remove("homes");
    assert!(canonical_profile(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut value = complete();
    value["inventory"][0]["item"]["material"] = json!("STONE");
    assert!(canonical_profile(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut value = complete();
    value["homes"][0]["x"] = json!(30_000_001.0);
    assert!(canonical_profile(&serde_json::to_vec(&value).unwrap()).is_err());
}
