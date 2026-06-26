mod helpers;
mod types;

use helpers::{finding, next_actions};
pub use types::*;

#[cfg(test)]
mod tests;

pub fn diagnose_network(
    input: NetworkDiagnosticInput,
    facts: NetworkDiagnosticFacts,
) -> NetworkDiagnosticReport {
    let (target_host, target_port) = facts
        .srv
        .first()
        .map(|srv| (srv.target.as_str(), srv.port))
        .unwrap_or((input.host.as_str(), input.port));
    let effective_target = format!("{target_host}:{target_port}");
    let mut findings = Vec::new();
    classify_srv(&input, &facts, target_host, &mut findings);
    classify_expected(&input, &facts, &mut findings);
    classify_tcp(&effective_target, &facts, &mut findings);
    classify_status(&effective_target, &facts, &mut findings);
    if !findings
        .iter()
        .any(|finding| finding.severity == NetworkSeverity::Blocking)
    {
        findings.push(finding(
            NetworkSeverity::Ok,
            NetworkFindingCode::Ready,
            "hostname path is ready or blocked only by warnings",
        ));
    }
    let next_actions = next_actions(&findings);
    NetworkDiagnosticReport {
        input,
        effective_target,
        srv: facts.srv,
        addresses: facts.addresses,
        findings,
        next_actions,
    }
}

fn classify_srv(
    input: &NetworkDiagnosticInput,
    facts: &NetworkDiagnosticFacts,
    target_host: &str,
    findings: &mut Vec<NetworkFinding>,
) {
    if facts.srv.is_empty() && input.port == 25565 {
        findings.push(finding(
            NetworkSeverity::Info,
            NetworkFindingCode::NoSrvDefaultPort,
            "no SRV record found; A or AAAA is enough on port 25565",
        ));
    }
    if let Some(srv) = facts.srv.first() {
        findings.push(finding(
            NetworkSeverity::Info,
            NetworkFindingCode::SrvSelected,
            format!("client selects SRV target {}:{}", srv.target, srv.port),
        ));
        if !facts
            .addresses
            .iter()
            .any(|record| record.host == target_host)
        {
            findings.push(finding(
                NetworkSeverity::Blocking,
                NetworkFindingCode::SrvTargetMissingAddress,
                "SRV target has no A or AAAA address record",
            ));
        }
    }
}

fn classify_expected(
    input: &NetworkDiagnosticInput,
    facts: &NetworkDiagnosticFacts,
    findings: &mut Vec<NetworkFinding>,
) {
    let Some(expected) = &input.expected_address else {
        return;
    };
    if !facts
        .addresses
        .iter()
        .any(|record| &record.address == expected)
    {
        findings.push(finding(
            NetworkSeverity::Warning,
            NetworkFindingCode::ExpectedAddressMissing,
            format!("resolved addresses do not include expected {expected}"),
        ));
    }
}

fn classify_tcp(target: &str, facts: &NetworkDiagnosticFacts, findings: &mut Vec<NetworkFinding>) {
    if facts
        .tcp
        .iter()
        .any(|check| check.target == target && check.ok)
    {
        return;
    }
    findings.push(finding(
        NetworkSeverity::Blocking,
        NetworkFindingCode::TcpUnavailable,
        format!("TCP connection to {target} failed"),
    ));
}

fn classify_status(
    target: &str,
    facts: &NetworkDiagnosticFacts,
    findings: &mut Vec<NetworkFinding>,
) {
    if facts
        .status_ping
        .iter()
        .any(|check| check.target == target && check.ok)
    {
        return;
    }
    findings.push(finding(
        NetworkSeverity::Warning,
        NetworkFindingCode::StatusPingFailed,
        format!("Minecraft status ping to {target} failed"),
    ));
    if facts
        .status_ping
        .iter()
        .any(|check| check.target.starts_with("direct:") && check.ok)
    {
        findings.push(finding(
            NetworkSeverity::Warning,
            NetworkFindingCode::DirectIpComparisonWorks,
            "direct IP status ping works but hostname status ping failed",
        ));
    }
}
