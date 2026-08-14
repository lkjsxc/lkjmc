#[cfg(test)]
use std::collections::BTreeMap;
use std::time::Duration;

use lkjmc_core::command::{CommandEnvelope, CommandErrorBody, CommandResponse};
use lkjmc_core::command_registry::CommandContract;

pub(crate) const ADMISSION_LIMIT: usize = 8;
pub(crate) const DEADLINE: Duration = Duration::from_secs(8);
pub(crate) const NETWORK_APPLY_DEADLINE: Duration = Duration::from_secs(20 * 60);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum EffectClass {
    DeniedUnproved,
    LocalObservation,
    NetworkApply,
    PrivateCredentialWrite,
    PostgresqlDesiredSet,
    PostgresqlRead,
    RestartRequired,
    RuntimeObservation,
}

pub(crate) fn enforce(
    request: CommandEnvelope,
    contract: &CommandContract,
    subject: &crate::authz::AuthenticatedSubject,
) -> Result<EffectClass, CommandResponse> {
    let class = classify(contract).ok_or_else(|| {
        error(
            request.clone(),
            "command.effect_denied",
            "command effect class is not recognized; no handler was invoked",
            false,
        )
    })?;
    match class {
        EffectClass::LocalObservation
        | EffectClass::PostgresqlRead
        | EffectClass::PostgresqlDesiredSet
        | EffectClass::RuntimeObservation => Ok(class),
        EffectClass::NetworkApply if subject.allows_local_runtime_effects() => Ok(class),
        EffectClass::NetworkApply => Err(error(
            request,
            "auth.local_peer_required",
            "network apply requires an authenticated local Unix peer; no effect was started",
            false,
        )),
        EffectClass::PrivateCredentialWrite if subject.allows_local_runtime_effects() => Ok(class),
        EffectClass::PrivateCredentialWrite => Err(error(
            request,
            "auth.local_peer_required",
            "credential creation requires an authenticated local Unix peer; no effect was started",
            false,
        )),
        EffectClass::RestartRequired => Err(error(
            request,
            "config.restart_required",
            "configuration applies only at daemon restart; no config was read or applied",
            false,
        )),
        EffectClass::DeniedUnproved => Err(error(
            request,
            "command.effect_denied",
            "command effect is unproved and was not started",
            false,
        )),
    }
}

pub(crate) fn classify(contract: &CommandContract) -> Option<EffectClass> {
    match contract.effect.as_str() {
        "denied-unproved" => Some(EffectClass::DeniedUnproved),
        "local-observation" => Some(EffectClass::LocalObservation),
        "network-apply" => Some(EffectClass::NetworkApply),
        "private-credential-write" => Some(EffectClass::PrivateCredentialWrite),
        "postgresql-desired-set" => Some(EffectClass::PostgresqlDesiredSet),
        "postgresql-read" => Some(EffectClass::PostgresqlRead),
        "restart-required" => Some(EffectClass::RestartRequired),
        "runtime-observation" => Some(EffectClass::RuntimeObservation),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) fn counts() -> BTreeMap<EffectClass, usize> {
    let mut counts = BTreeMap::new();
    for contract in lkjmc_core::command_registry::all() {
        if let Some(class) = classify(contract) {
            *counts.entry(class).or_default() += 1;
        }
    }
    counts
}

fn error(request: CommandEnvelope, code: &str, message: &str, retryable: bool) -> CommandResponse {
    CommandResponse {
        request_id: request.request_id,
        ok: false,
        body: None,
        error: Some(CommandErrorBody {
            code: code.to_string(),
            message: message.to_string(),
            retryable,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_apply_requires_a_local_unix_subject() -> Result<(), String> {
        let contract = lkjmc_core::command_registry::contract_for("bootstrap.apply")
            .ok_or("bootstrap.apply contract missing")?;
        let request = CommandEnvelope {
            request_id: lkjmc_core::id::CommandId::internal("network-apply-policy-test"),
            actor: lkjmc_core::command::Actor {
                kind: lkjmc_core::command::ActorKind::Cli,
                name: "untrusted".to_string(),
            },
            command: "bootstrap.apply".to_string(),
            body: serde_json::json!({"acceptMinecraftEula": true}),
        };
        assert_eq!(
            enforce(
                request.clone(),
                contract,
                &crate::authz::AuthenticatedSubject::unix_peer(
                    crate::transport::peer::verified_unix_peer_for_test(1000),
                )
            )
            .map_err(|_| "local network apply denied")?,
            EffectClass::NetworkApply
        );
        assert!(enforce(
            request.clone(),
            contract,
            &crate::authz::AuthenticatedSubject::internal(),
        )
        .is_err());
        let remote_cli = crate::authz::AuthenticatedSubject::credential(
            lkjmc_store::daemon_token::DaemonTokenRecord {
                credential_id: uuid::Uuid::nil(),
                surface: "cli".to_string(),
                principal_kind: "operator".to_string(),
                principal_id: "remote-test".to_string(),
                scopes: vec!["lkjmc.admin.operator".to_string()],
                expires_at_micros: 1,
            },
        );
        let denied = enforce(request, contract, &remote_cli)
            .err()
            .ok_or("remote network apply unexpectedly admitted")?;
        assert_eq!(
            denied.error.as_ref().map(|error| error.code.as_str()),
            Some("auth.local_peer_required")
        );
        Ok(())
    }

    #[test]
    fn private_credential_write_requires_local_unix_peer() -> Result<(), String> {
        let contract = lkjmc_core::command_registry::contract_for("security.daemon-token.create")
            .ok_or("credential contract missing")?;
        let request = CommandEnvelope {
            request_id: lkjmc_core::id::CommandId::internal("credential-policy-test"),
            actor: lkjmc_core::command::Actor {
                kind: lkjmc_core::command::ActorKind::Cli,
                name: "test".into(),
            },
            command: "security.daemon-token.create".into(),
            body: serde_json::json!({}),
        };
        assert_eq!(
            enforce(
                request.clone(),
                contract,
                &crate::authz::AuthenticatedSubject::unix_peer(
                    crate::transport::peer::verified_unix_peer_for_test(1000),
                ),
            )
            .map_err(|_| "local credential write denied")?,
            EffectClass::PrivateCredentialWrite
        );
        let remote = crate::authz::AuthenticatedSubject::credential(
            lkjmc_store::daemon_token::DaemonTokenRecord {
                credential_id: uuid::Uuid::nil(),
                surface: "cli".into(),
                principal_kind: "operator".into(),
                principal_id: "remote".into(),
                scopes: vec!["lkjmc.admin.admin".into()],
                expires_at_micros: i64::MAX,
            },
        );
        let denied = enforce(request, contract, &remote)
            .err()
            .ok_or("remote credential write admitted")?;
        assert_eq!(
            denied.error.as_ref().map(|error| error.code.as_str()),
            Some("auth.local_peer_required")
        );
        Ok(())
    }

    #[test]
    fn effect_classes_enforced() {
        let counts = counts();
        let contracts = lkjmc_core::command_registry::all();
        assert_eq!(counts.values().sum::<usize>(), contracts.len());
        assert_eq!(counts.get(&EffectClass::LocalObservation), Some(&1));
        assert_eq!(counts.get(&EffectClass::NetworkApply), Some(&1));
        assert_eq!(counts.get(&EffectClass::PostgresqlRead), Some(&2));
        assert_eq!(counts.get(&EffectClass::PrivateCredentialWrite), Some(&1));
        assert_eq!(counts.get(&EffectClass::PostgresqlDesiredSet), Some(&3));
        assert_eq!(counts.get(&EffectClass::RestartRequired), Some(&1));
        assert_eq!(counts.get(&EffectClass::RuntimeObservation), Some(&3));
        assert_eq!(
            counts.get(&EffectClass::DeniedUnproved),
            Some(&(contracts.len() - 12))
        );
        assert!(contracts
            .iter()
            .all(|contract| classify(contract).is_some()));
    }
}
