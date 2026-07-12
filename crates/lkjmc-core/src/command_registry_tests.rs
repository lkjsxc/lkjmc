use std::collections::BTreeSet;

use serde_json::json;

use super::*;

#[path = "command_shape_tests.rs"]
mod command_shape_tests;

const AUTH: &[&str] = &["admin", "open", "operator", "player"];
const SURFACES: &[&str] = &["cli", "internal", "web"];

#[test]
fn registry_is_sorted_unique_and_closed() {
    let names = all()
        .iter()
        .map(|entry| entry.name.as_str())
        .collect::<Vec<_>>();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    assert_eq!(names, sorted);
    assert_eq!(
        names.len(),
        names.iter().copied().collect::<BTreeSet<_>>().len()
    );
    assert!(!names.is_empty());
}

#[test]
fn every_contract_rejects_unknown_body_members() {
    for contract in all() {
        let result = validate_body(&contract.name, &json!({"contractProbe": true}));
        assert!(
            result.is_err(),
            "{} accepted an unknown member",
            contract.name
        );
    }
}

#[test]
fn registry_uses_checked_vocabulary() {
    for entry in all() {
        assert!(
            AUTH.contains(&entry.authorization.as_str()),
            "{}",
            entry.name
        );
        assert!(!entry.surfaces.is_empty(), "{}", entry.name);
        assert!(entry
            .surfaces
            .iter()
            .all(|item| SURFACES.contains(&item.as_str())));
        assert_eq!(entry.identity, "transport-subject");
        assert_eq!(entry.response.envelope, "command-response-v1");
        match &entry.request {
            RequestContract::Fields(request) => {
                assert!(!request.fields.is_empty(), "{}", entry.name);
                assert!(request
                    .required_any_of
                    .iter()
                    .all(|group| group.iter().all(|field| request.fields.contains_key(field))));
            }
            RequestContract::HandlerDefined(request) => {
                assert_eq!(request.body, "handler-defined", "{}", entry.name);
            }
        }
    }
}

#[test]
fn every_contract_enforces_its_required_members_and_types() {
    for contract in all() {
        let RequestContract::Fields(request) = &contract.request else {
            assert!(validate_body(&contract.name, &json!({})).is_ok());
            continue;
        };
        let mut body = serde_json::Map::new();
        for (name, field) in &request.fields {
            if field.required {
                body.insert(name.clone(), sample(&field.value_type));
            }
        }
        for group in &request.required_any_of {
            if let Some(name) = group.first() {
                if let Some(field) = request.fields.get(name) {
                    body.insert(name.clone(), sample(&field.value_type));
                }
            }
        }
        assert!(
            validate_body(&contract.name, &json!(body)).is_ok(),
            "{}",
            contract.name
        );
        for (name, field) in &request.fields {
            if field.required {
                let mut missing = body.clone();
                missing.remove(name);
                assert!(
                    validate_body(&contract.name, &json!(missing)).is_err(),
                    "{}",
                    contract.name
                );
            }
            let mut wrong = body.clone();
            wrong.insert(name.clone(), json!([]));
            if field.value_type != ValueType::Array {
                assert!(
                    validate_body(&contract.name, &json!(wrong)).is_err(),
                    "{}",
                    contract.name
                );
            }
        }
    }
}

#[test]
fn real_cli_and_web_payload_literals_match_their_contracts() -> Result<(), String> {
    let cli_asset = include_str!("../../lkjmc-cli/src/commands_asset.rs");
    let cli_root = include_str!("../../lkjmc-cli/src/commands.rs");
    let web = include_str!("../../lkjmc-daemon/src/web/api.rs");
    assert!(cli_asset.contains("json!({\"plugin\": plugin})"));
    assert!(cli_root.contains("\"audit.tail\",\n            json!({\"lines\": lines})"));
    assert!(web.contains("\"audit.tail\", json!({\"lines\": 50})"));
    for (command, body) in [
        ("asset.plugin.inspect", json!({"plugin":"viaversion"})),
        ("asset.plugin.sync", json!({"plugin":"viaversion"})),
        ("audit.tail", json!({"lines":50})),
        ("instance.start", json!({"id":"hub"})),
        ("status", json!({})),
    ] {
        validate_body(command, &body)?;
    }
    Ok(())
}

#[test]
fn lookup_returns_known_command() -> Result<(), String> {
    let status = contract_for("status").ok_or_else(|| "status contract".to_string())?;
    assert_eq!(status.name, "status");
    validate_body("status", &json!({}))
}

fn sample(value_type: &ValueType) -> serde_json::Value {
    match value_type {
        ValueType::Array => json!([]),
        ValueType::Boolean => json!(true),
        ValueType::EmptyObject => json!({}),
        ValueType::Integer => json!(1),
        ValueType::Number => json!(1.5),
        ValueType::RconConfig => json!({"password":"secret","port":25575}),
        ValueType::ShopMetadata => json!({}),
        ValueType::String => json!("value"),
        ValueType::WorldLocation => json!({"world":"world","x":1.0,"y":64.0,"z":1.0}),
    }
}
