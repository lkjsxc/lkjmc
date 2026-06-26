use super::{NetworkFinding, NetworkFindingCode, NetworkSeverity};

pub fn finding(
    severity: NetworkSeverity,
    code: NetworkFindingCode,
    message: impl Into<String>,
) -> NetworkFinding {
    NetworkFinding {
        severity,
        code,
        message: message.into(),
    }
}

pub fn next_actions(findings: &[NetworkFinding]) -> Vec<String> {
    findings
        .iter()
        .filter(|finding| finding.severity != NetworkSeverity::Ok)
        .map(|finding| finding.message.clone())
        .collect()
}
