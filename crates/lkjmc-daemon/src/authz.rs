use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope, CommandResponse};
use lkjmc_core::command_registry::CommandContract;

use crate::app::AppState;
use crate::dispatch as api;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedSubject {
    pub surface: String,
    pub principal_kind: Option<String>,
    pub principal_id: Option<String>,
    pub verified_permissions: Vec<String>,
    local_peer: bool,
    internal: bool,
    audit_name: String,
}

impl AuthenticatedSubject {
    pub fn credential(record: lkjmc_store::daemon_token::DaemonTokenRecord) -> Self {
        Self {
            surface: record.surface,
            principal_kind: Some(record.principal_kind),
            principal_id: Some(record.principal_id),
            verified_permissions: record.scopes,
            local_peer: false,
            internal: false,
            audit_name: format!("credential:{}", record.credential_id),
        }
    }

    pub fn web_session() -> Self {
        Self {
            surface: "web".into(),
            principal_kind: Some("operator".into()),
            principal_id: Some("browser-session".into()),
            verified_permissions: vec!["lkjmc.admin.admin".into(), "lkjmc.admin.operator".into()],
            local_peer: false,
            internal: false,
            audit_name: "web-session".into(),
        }
    }

    pub(crate) fn unix_peer(peer: crate::transport::peer::VerifiedUnixPeer) -> Self {
        let uid = peer.uid();
        Self {
            surface: "cli".into(),
            principal_kind: Some("operator".into()),
            principal_id: Some(format!("uid:{uid}")),
            verified_permissions: Vec::new(),
            local_peer: true,
            internal: false,
            audit_name: format!("unix-peer:{uid}"),
        }
    }

    pub fn internal() -> Self {
        Self {
            surface: "internal".into(),
            principal_kind: None,
            principal_id: None,
            verified_permissions: Vec::new(),
            local_peer: false,
            internal: true,
            audit_name: "internal".into(),
        }
    }

    pub(crate) fn event_actor(&self) -> Actor {
        let kind = match self.surface.as_str() {
            "cli" => ActorKind::Cli,
            "web" => ActorKind::WebOperator,
            "discord" => ActorKind::Discord,
            "paper" => ActorKind::PaperPlugin,
            "velocity" => ActorKind::VelocityPlugin,
            _ => ActorKind::Daemon,
        };
        Actor {
            kind,
            name: self.audit_name.clone(),
        }
    }

    fn allows(&self, permission: &str) -> bool {
        self.internal
            || self.local_peer
            || self
                .verified_permissions
                .iter()
                .any(|value| value == permission || value == "lkjmc.admin.admin")
    }

    pub fn allows_sync_read(&self) -> bool {
        matches!(self.surface.as_str(), "paper" | "velocity")
            && self
                .verified_permissions
                .iter()
                .any(|value| value == "lkjmc.sync.read")
    }

    pub(crate) fn allows_local_runtime_effects(&self) -> bool {
        self.local_peer
    }

    pub(crate) fn allows_observability(&self) -> bool {
        self.internal
            || self.local_peer
            || self
                .verified_permissions
                .iter()
                .any(|value| matches!(value.as_str(), "lkjmc.admin.admin" | "lkjmc.admin.operator"))
    }

    fn supports(&self, contract: &CommandContract) -> bool {
        self.internal
            || contract
                .surfaces
                .iter()
                .any(|surface| surface == &self.surface)
    }

    fn bind(&self, request: &mut CommandEnvelope) -> bool {
        if self.internal {
            return true;
        }
        let kind = match self.surface.as_str() {
            "cli" => ActorKind::Cli,
            "web" => ActorKind::WebOperator,
            _ => return false,
        };
        request.actor = Actor {
            kind,
            name: self.audit_name.clone(),
        };
        true
    }
}

pub fn authorize(
    state: &AppState,
    mut request: CommandEnvelope,
    contract: &CommandContract,
    subject: &AuthenticatedSubject,
) -> Result<CommandEnvelope, CommandResponse> {
    if !subject.supports(contract) || !subject.bind(&mut request) {
        crate::security_audit::denial(state, &subject.surface, "surface-denied");
        return Err(api::error(
            request,
            "auth.surface_denied",
            "credential is unavailable for this command surface",
            false,
        ));
    }
    let permission = permission_for(&contract.authorization);
    if subject.allows(permission) {
        return Ok(request);
    }
    crate::security_audit::denial(state, &subject.surface, "policy-denied");
    Err(api::error(
        request,
        "auth.policy_denied",
        "credential scope does not allow this command",
        false,
    ))
}

fn permission_for(authorization: &str) -> &'static str {
    match authorization {
        "admin" => "lkjmc.admin.admin",
        "operator" => "lkjmc.admin.operator",
        _ => "lkjmc.player.self",
    }
}

#[cfg(test)]
#[path = "authz_tests.rs"]
mod authz_tests;
