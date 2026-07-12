use std::collections::BTreeMap;

#[test]
fn daemon_registrations_match_command_contract() {
    let mut registered = BTreeMap::new();
    for entry in crate::dispatch::registrations() {
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
fn command_contract_authorization_has_a_closed_policy_class() {
    for contract in lkjmc_core::command_registry::all() {
        assert!(
            matches!(
                contract.authorization.as_str(),
                "admin" | "operator" | "player"
            ),
            "{}",
            contract.name
        );
    }
}
