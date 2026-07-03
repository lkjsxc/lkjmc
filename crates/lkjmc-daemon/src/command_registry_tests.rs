use std::collections::BTreeMap;

#[test]
fn daemon_registrations_match_command_contract() {
    let mut registered = BTreeMap::new();
    for entry in crate::api::registrations() {
        *registered.entry(entry.name).or_insert(0_usize) += 1;
    }
    for contract in lkjmc_core::command_registry::all() {
        assert_eq!(
            registered.remove(contract.name.as_str()),
            Some(1),
            "{}",
            contract.name
        );
    }
    assert!(registered.is_empty(), "extra registrations: {registered:?}");
}

#[test]
fn command_contract_authorization_matches_authz_table() {
    for contract in lkjmc_core::command_registry::all() {
        let expected = if crate::authz::required(&contract.name).is_some() {
            "admin"
        } else {
            "open"
        };
        assert_eq!(contract.authorization, expected, "{}", contract.name);
    }
}
