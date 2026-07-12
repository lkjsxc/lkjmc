#[cfg(test)]
use std::collections::BTreeMap;
use std::time::Duration;

use lkjmc_core::command::{CommandEnvelope, CommandErrorBody, CommandResponse};
use lkjmc_core::command_registry::CommandContract;

pub(crate) const ADMISSION_LIMIT: usize = 8;
pub(crate) const DEADLINE: Duration = Duration::from_secs(8);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum EffectClass {
    DeniedUnproved,
    LocalObservation,
    PostgresqlDesiredSet,
    PostgresqlRead,
    RestartRequired,
}

pub(crate) fn enforce(
    request: CommandEnvelope,
    contract: &CommandContract,
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
        | EffectClass::PostgresqlDesiredSet => Ok(class),
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
        "postgresql-desired-set" => Some(EffectClass::PostgresqlDesiredSet),
        "postgresql-read" => Some(EffectClass::PostgresqlRead),
        "restart-required" => Some(EffectClass::RestartRequired),
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
    fn effect_classes_enforced() {
        let counts = counts();
        assert_eq!(counts.values().sum::<usize>(), 137);
        assert_eq!(counts.get(&EffectClass::LocalObservation), Some(&1));
        assert_eq!(counts.get(&EffectClass::PostgresqlRead), Some(&2));
        assert_eq!(counts.get(&EffectClass::PostgresqlDesiredSet), Some(&2));
        assert_eq!(counts.get(&EffectClass::RestartRequired), Some(&1));
        assert_eq!(counts.get(&EffectClass::DeniedUnproved), Some(&131));
        assert!(lkjmc_core::command_registry::all()
            .iter()
            .all(|contract| classify(contract).is_some()));
    }
}
