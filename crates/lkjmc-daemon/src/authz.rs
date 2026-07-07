use lkjmc_core::command::{CommandEnvelope, CommandResponse};

use crate::app::AppState;
use crate::dispatch as api;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedSubject {
    pub surface: String,
    pub root: bool,
    pub verified_permissions: Vec<String>,
}

impl AuthenticatedSubject {
    pub fn root(surface: &'static str) -> Self {
        Self {
            surface: surface.to_string(),
            root: true,
            verified_permissions: Vec::new(),
        }
    }

    pub fn scoped(surface: impl Into<String>, scopes: Vec<String>) -> Self {
        Self {
            surface: surface.into(),
            root: false,
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

pub fn required(command: &str) -> Option<&'static str> {
    match command {
        "status" | "doctor" => Some("lkjmc.admin.status"),
        "config.reload" => Some("lkjmc.admin.reload"),
        "instance.list" => Some("lkjmc.admin.instance.list"),
        "instance.create" => Some("lkjmc.admin.instance.create"),
        "instance.start" => Some("lkjmc.admin.instance.start"),
        "instance.stop" => Some("lkjmc.admin.instance.stop"),
        "instance.restart" => Some("lkjmc.admin.instance.restart"),
        "instance.delete" => Some("lkjmc.admin.instance.delete"),
        "instance.wake.cleanup" => Some("lkjmc.admin.instance.start"),
        "economy.catalog.seed-defaults" | "shop.item.upsert" => Some("lkjmc.admin.economy"),
        "admin.grant.create" | "admin.grant.revoke" | "admin.audit.tail" => {
            Some("lkjmc.admin.admin")
        }
        "adventure.session.list" | "adventure.session.cancel" => Some("lkjmc.admin.instance.list"),
        command if command.starts_with("security.daemon-token.") => Some("lkjmc.admin.admin"),
        _ => None,
    }
}

pub fn enforce(
    state: &AppState,
    request: &CommandEnvelope,
    permission: &str,
    subject: &AuthenticatedSubject,
) -> Option<CommandResponse> {
    if subject.allows(permission) || grant_allowed(state, request, permission).unwrap_or(false) {
        return None;
    }
    Some(api::error(
        request.clone(),
        "admin.denied",
        "admin permission denied",
        false,
    ))
}

fn grant_allowed(
    state: &AppState,
    request: &CommandEnvelope,
    permission: &str,
) -> Result<bool, String> {
    if state.database_url().is_none() {
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
    if permissions
        .iter()
        .any(|value| value == permission || value == "lkjmc.admin.admin")
    {
        return Ok(true);
    }
    let _ = lkjmc_store::admin::insert_audit(
        &mut client,
        "daemon",
        "authz",
        "admin.denied",
        kind,
        id,
        "denied",
    );
    Ok(false)
}

#[cfg(test)]
#[path = "authz_tests.rs"]
mod authz_tests;
