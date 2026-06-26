use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkDiagnosticInput {
    pub host: String,
    pub port: u16,
    pub expected_address: Option<String>,
    pub direct_address: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SrvRecord {
    pub target: String,
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddressRecord {
    pub host: String,
    pub address: String,
    pub family: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReachabilityCheck {
    pub target: String,
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkDiagnosticFacts {
    pub srv: Vec<SrvRecord>,
    pub addresses: Vec<AddressRecord>,
    pub tcp: Vec<ReachabilityCheck>,
    pub status_ping: Vec<ReachabilityCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkDiagnosticReport {
    pub input: NetworkDiagnosticInput,
    pub effective_target: String,
    pub srv: Vec<SrvRecord>,
    pub addresses: Vec<AddressRecord>,
    pub findings: Vec<NetworkFinding>,
    pub next_actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkFinding {
    pub severity: NetworkSeverity,
    pub code: NetworkFindingCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkSeverity {
    Ok,
    Info,
    Warning,
    Blocking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkFindingCode {
    NoSrvDefaultPort,
    SrvSelected,
    SrvTargetMissingAddress,
    ExpectedAddressMissing,
    TcpUnavailable,
    StatusPingFailed,
    DirectIpComparisonWorks,
    Ready,
}
