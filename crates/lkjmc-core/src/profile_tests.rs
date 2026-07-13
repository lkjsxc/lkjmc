use serde_json::{json, Value};

use crate::profile_validation::canonical_profile;

type TestResult = Result<(), String>;

fn encode(value: &Value) -> Result<Vec<u8>, String> {
    serde_json::to_vec(value).map_err(|error| error.to_string())
}

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
fn profile_format_safe_complete_and_canonical() -> TestResult {
    let input = serde_json::to_vec_pretty(&complete()).map_err(|error| error.to_string())?;
    let first = canonical_profile(&input)?;
    let second = canonical_profile(&first.json)?;
    assert_eq!(first.json, second.json);
    assert_eq!(first.sha256, second.sha256);
    let canonical = String::from_utf8(first.json).map_err(|error| error.to_string())?;
    assert!(!canonical.contains(" \n"));
    Ok(())
}

#[test]
fn profile_rejects_unknown_duplicate_unsafe_and_incomplete() -> TestResult {
    let encoded = serde_json::to_string(&complete()).map_err(|error| error.to_string())?;
    let duplicate = encoded.replacen(
        "\"schema\":\"lkjmc-profile-one\"",
        "\"schema\":\"lkjmc-profile-one\",\"schema\":\"other\"",
        1,
    );
    let error = match canonical_profile(duplicate.as_bytes()) {
        Err(error) => error,
        Ok(_) => return Err("duplicate field was accepted".into()),
    };
    assert!(error.contains("duplicate field `schema`"));
    let mut value = complete();
    value
        .as_object_mut()
        .ok_or_else(|| "profile is not an object".to_string())?
        .insert("opaque".into(), json!("bytes"));
    assert!(canonical_profile(&encode(&value)?).is_err());
    let mut value = complete();
    value
        .as_object_mut()
        .ok_or_else(|| "profile is not an object".to_string())?
        .remove("homes");
    assert!(canonical_profile(&encode(&value)?).is_err());
    let mut value = complete();
    value["inventory"][0]["item"]["material"] = json!("STONE");
    assert!(canonical_profile(&encode(&value)?).is_err());
    let mut value = complete();
    value["homes"][0]["x"] = json!(30_000_001.0);
    assert!(canonical_profile(&encode(&value)?).is_err());
    let nonfinite = encoded.replace("\"progress\":0.5", "\"progress\":1e999");
    assert!(canonical_profile(nonfinite.as_bytes()).is_err());
    let mut value = complete();
    value["vitals"]["food"] = json!(21);
    assert!(canonical_profile(&encode(&value)?).is_err());
    Ok(())
}

#[test]
fn profile_rejects_oversize_collections_and_strings() -> TestResult {
    for (field, count, entry) in [
        ("potionEffects", 129, potion("speed")),
        ("pluginData", 129, json!({"key":"lkjmc:key","value":"v"})),
        ("achievements", 1025, json!("lkjmc:achievement")),
    ] {
        let mut value = complete();
        value[field] = Value::Array((0..count).map(|index| numbered(&entry, index)).collect());
        assert!(canonical_profile(&encode(&value)?).is_err());
    }
    let mut value = complete();
    value["inventory"][0]["item"]["customName"] = json!("x".repeat(1025));
    assert!(canonical_profile(&encode(&value)?).is_err());
    assert!(canonical_profile(&vec![b' '; 1_048_577]).is_err());
    Ok(())
}

#[test]
fn profile_canonicalizes_set_like_order() -> TestResult {
    let mut first = complete();
    first["inventory"] = json!([
        {"slot":2,"item":item("minecraft:stone")},
        {"slot":0,"item":item("minecraft:dirt")}
    ]);
    first["pluginData"] = json!([
        {"key":"lkjmc:z","value":"z"}, {"key":"lkjmc:a","value":"a"}
    ]);
    let mut second = first.clone();
    second["inventory"]
        .as_array_mut()
        .ok_or_else(|| "inventory is not an array".to_string())?
        .reverse();
    second["pluginData"]
        .as_array_mut()
        .ok_or_else(|| "pluginData is not an array".to_string())?
        .reverse();
    let first = canonical_profile(&encode(&first)?)?;
    let second = canonical_profile(&encode(&second)?)?;
    assert_eq!(first.json, second.json);
    assert_eq!(first.sha256, second.sha256);
    Ok(())
}

fn potion(id: &str) -> Value {
    json!({"id":format!("minecraft:{id}"),"amplifier":0,"durationTicks":20,
        "ambient":false,"particles":true,"icon":true})
}

fn numbered(entry: &Value, index: usize) -> Value {
    match entry {
        Value::String(_) => json!(format!("lkjmc:achievement_{index}")),
        Value::Object(object) if object.contains_key("key") => {
            json!({"key":format!("lkjmc:key_{index}"),"value":"v"})
        }
        _ => potion(&format!("effect_{index}")),
    }
}
