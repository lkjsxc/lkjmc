use serde_json::json;

use super::canonical_request_profile;

fn profile() -> serde_json::Value {
    json!({
        "schema":"lkjmc-profile-one","inventory":[],"armor":[],"offhand":null,
        "selectedHotbarSlot":0,"enderChest":[],
        "experience":{"progress":0.0,"level":0,"total":0},
        "vitals":{"health":20.0,"food":20,"saturation":5.0,"air":300},
        "potionEffects":[],"gameMode":null,"pluginData":[],"homes":[],"warps":[],
        "points":0,"achievements":[],
        "settings":{"menuEnabled":true,"hudEnabled":true,"tipsEnabled":true,"privacy":"private"},
        "language":"en"
    })
}

#[test]
fn daemon_canonicalizes_and_computes_profile_integrity() {
    let profile_json = serde_json::to_string_pretty(&profile()).unwrap();
    let body = json!({"profileJson": profile_json, "sha256": "caller-value-is-ignored"});
    let result = canonical_request_profile(&body).unwrap();
    assert_eq!(result.sha256.len(), 64);
    assert_eq!(
        result.sha256,
        lkjmc_core::profile_validation::canonical_profile(&result.json)
            .unwrap()
            .sha256
    );
}

#[test]
fn daemon_preserves_raw_json_for_duplicate_rejection() {
    let encoded = serde_json::to_string(&profile()).unwrap();
    let duplicate = encoded.replacen(
        "\"schema\":\"lkjmc-profile-one\"",
        "\"schema\":\"lkjmc-profile-one\",\"schema\":\"other\"",
        1,
    );
    let error = canonical_request_profile(&json!({"profileJson": duplicate}))
        .err()
        .unwrap();
    assert!(error.contains("duplicate field `schema`"));
}
