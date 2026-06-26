use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootstrapDiagnostic {
    pub severity: DiagnosticSeverity,
    pub code: DiagnosticCode,
    pub message: String,
    pub target: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticSeverity {
    Blocking,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticCode {
    MinecraftEulaRequired,
    DatabaseUnavailable,
    JavaPortUnavailable,
    BackendPortUnavailable,
    UnmanagedDirectoryConflict,
    BedrockWithdrawn,
    BedrockBlocked,
    ViaWithdrawn,
    ViaBackwardsDependency,
    PortReallocated,
}

impl BootstrapDiagnostic {
    pub fn blocking(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Blocking,
            code,
            message: message.into(),
            target: None,
        }
    }

    pub fn warning(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            code,
            message: message.into(),
            target: None,
        }
    }

    pub fn info(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Info,
            code,
            message: message.into(),
            target: None,
        }
    }
}
