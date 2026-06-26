use super::*;

#[test]
fn selects_srv_target_and_reports_missing_address() {
    let report = diagnose_network(
        input(),
        NetworkDiagnosticFacts {
            srv: vec![SrvRecord {
                target: "srv.lkjsxc.com".to_string(),
                port: 25566,
            }],
            addresses: vec![address("lkjsxc.com", "192.168.1.2")],
            tcp: vec![],
            status_ping: vec![],
        },
    );
    assert_eq!(report.effective_target, "srv.lkjsxc.com:25566");
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.code == NetworkFindingCode::SrvTargetMissingAddress));
}

#[test]
fn treats_missing_srv_on_default_port_as_info() {
    let report = diagnose_network(
        input(),
        NetworkDiagnosticFacts {
            srv: vec![],
            addresses: vec![address("lkjsxc.com", "192.168.1.2")],
            tcp: vec![ok("lkjsxc.com:25565")],
            status_ping: vec![ok("lkjsxc.com:25565")],
        },
    );
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.code == NetworkFindingCode::NoSrvDefaultPort));
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.code == NetworkFindingCode::Ready));
}

fn input() -> NetworkDiagnosticInput {
    NetworkDiagnosticInput {
        host: "lkjsxc.com".to_string(),
        port: 25565,
        expected_address: Some("192.168.1.2".to_string()),
        direct_address: Some("192.168.1.2".to_string()),
    }
}

fn address(host: &str, address: &str) -> AddressRecord {
    AddressRecord {
        host: host.to_string(),
        address: address.to_string(),
        family: "ipv4".to_string(),
    }
}

fn ok(target: &str) -> ReachabilityCheck {
    ReachabilityCheck {
        target: target.to_string(),
        ok: true,
        message: "ok".to_string(),
    }
}
