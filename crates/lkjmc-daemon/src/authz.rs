use lkjmc_core::command::{ActorKind, CommandEnvelope, CommandResponse};

use crate::app::AppState;
use crate::dispatch as api;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedSubject {
    pub surface: String,
    pub root: bool,
    pub principal_kind: Option<String>,
    pub principal_id: Option<String>,
    pub verified_permissions: Vec<String>,
}

impl AuthenticatedSubject {
    pub fn root(surface: &'static str) -> Self {
        Self {
            surface: surface.into(),
            root: true,
            principal_kind: None,
            principal_id: None,
            verified_permissions: vec![],
        }
    }

    pub fn scoped(
        surface: impl Into<String>,
        principal_kind: impl Into<String>,
        principal_id: impl Into<String>,
        scopes: Vec<String>,
    ) -> Self {
        Self {
            surface: surface.into(),
            root: false,
            principal_kind: Some(principal_kind.into()),
            principal_id: Some(principal_id.into()),
            verified_permissions: scopes,
        }
    }

    fn allows(&self, permission: &str) -> bool {
        self.root
            || self
                .verified_permissions
                .iter()
                .any(|value| value == permission || value == "lkjmc.admin.admin")
    }
}

pub fn required(command: &str) -> Option<&str> {
    Some(match command {
        "status" | "doctor" => "lkjmc.admin.status",
        "config.reload" => "lkjmc.admin.reload",
        "instance.list" => "lkjmc.admin.instance.list",
        "instance.create" => "lkjmc.admin.instance.create",
        "instance.start" => "lkjmc.admin.instance.start",
        "instance.stop" => "lkjmc.admin.instance.stop",
        "instance.restart" => "lkjmc.admin.instance.restart",
        "instance.delete" => "lkjmc.admin.instance.delete",
        "instance.wake.cleanup" => "lkjmc.admin.instance.start",
        "economy.catalog.seed-defaults" | "shop.item.upsert" => "lkjmc.admin.economy",
        "admin.grant.create" | "admin.grant.revoke" | "admin.audit.tail" => "lkjmc.admin.admin",
        "adventure.session.list" | "adventure.session.cancel" => "lkjmc.admin.instance.list",
        command if command.starts_with("security.") => "lkjmc.admin.admin",
        _ => return None,
    })
}

pub fn enforce(
    state: &AppState,
    request: &CommandEnvelope,
    permission: &str,
    subject: &AuthenticatedSubject,
) -> Option<CommandResponse> {
    if !subject_matches_request(request, subject) {
        return Some(api::error(
            request.clone(),
            "auth.subject_denied",
            "credential does not bind this surface or principal",
            false,
        ));
    }
    if subject.allows(permission)
        || grant_allowed(state, request, permission, subject).unwrap_or(false)
    {
        return None;
    }
    Some(api::error(
        request.clone(),
        "admin.denied",
        "admin permission denied",
        false,
    ))
}

fn subject_matches_request(request: &CommandEnvelope, subject: &AuthenticatedSubject) -> bool {
    if subject.root {
        return match subject.surface.as_str() {
            "bearer" => request.actor.kind == ActorKind::Cli,
            "web-session" | "web-bearer" => request.actor.kind == ActorKind::WebOperator,
            "local" | "internal" => true,
            _ => false,
        };
    }
    let surface = match request.actor.kind {
        ActorKind::PaperPlugin => "paper",
        ActorKind::VelocityPlugin => "velocity",
        ActorKind::Discord => "discord",
        ActorKind::WebOperator => "web",
        ActorKind::Cli => "cli",
        _ => return false,
    };
    subject.surface == surface
        && request
            .body
            .get("principalKind")
            .and_then(serde_json::Value::as_str)
            == subject.principal_kind.as_deref()
        && request
            .body
            .get("principalId")
            .and_then(serde_json::Value::as_str)
            == subject.principal_id.as_deref()
}

fn grant_allowed(
    state: &AppState,
    request: &CommandEnvelope,
    permission: &str,
    subject: &AuthenticatedSubject,
) -> Result<bool, String> {
    if state.database_url().is_none() || !subject.root {
        return Ok(false);
    }
    let kind = request
        .body
        .get("principalKind")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("minecraft-player");
    let Some(id) = request
        .body
        .get("principalId")
        .and_then(serde_json::Value::as_str)
    else {
        return Ok(false);
    };
    let mut client = state.database_connection()?;
    let permissions = lkjmc_store::admin::effective_permissions(&mut client, kind, id)
        .map_err(|error| error.to_string())?;
    Ok(permissions
        .iter()
        .any(|value| value == permission || value == "lkjmc.admin.admin"))
}

#[cfg(test)]
#[path = "authz_tests.rs"]
mod authz_tests;
