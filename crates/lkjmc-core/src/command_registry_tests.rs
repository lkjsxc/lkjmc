use std::collections::BTreeSet;

use serde_json::json;

use super::*;

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
        assert!(entry
            .request
            .required
            .iter()
            .all(|item| !entry.request.optional.contains(item)));
    }
}

#[test]
fn lookup_returns_known_command() -> Result<(), String> {
    let status = contract_for("status").ok_or_else(|| "status contract".to_string())?;
    assert_eq!(status.name, "status");
    validate_body("status", &json!({}))
}
