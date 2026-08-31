use serde::Serialize;

use crate::error::{OpsError, Result};
use crate::journal::{
    classify_recovery, BackupClosure, DeploymentJournal, MigrationIdentity, OperationPhase,
    RecoveryDecision,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeployOutcome {
    NoOp,
    Accepted,
    Abandoned,
    RolledBack,
    RestoreRequired,
    RecoveryBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployReceipt {
    pub schema_version: u32,
    pub operation_id: uuid::Uuid,
    pub outcome: DeployOutcome,
    pub from_commit: String,
    pub to_commit: String,
    pub manifest_sha256: String,
    pub phase: OperationPhase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_manifest_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_failure: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_decision: Option<RecoveryDecision>,
}

impl DeployReceipt {
    pub fn no_op(journal: &DeploymentJournal) -> Result<Self> {
        let mut completed = journal.clone();
        completed.transition(OperationPhase::NoOp)?;
        completed.transition(OperationPhase::Accepted)?;
        Ok(Self::from_journal(&completed, DeployOutcome::NoOp))
    }

    pub fn abandoned(journal: &DeploymentJournal) -> Result<Self> {
        journal.validate()?;
        if journal.phase != OperationPhase::Abandoned {
            return Err(OpsError::message(
                "abandoned receipt requires an abandoned deployment journal",
            ));
        }
        Ok(Self::from_journal(journal, DeployOutcome::Abandoned))
    }

    fn from_journal(journal: &DeploymentJournal, outcome: DeployOutcome) -> Self {
        Self {
            schema_version: 1,
            operation_id: journal.operation_id,
            outcome,
            from_commit: journal.source_commit.clone(),
            to_commit: journal.target_commit.clone(),
            manifest_sha256: journal.manifest_sha256.clone(),
            phase: journal.phase,
            backup_manifest_sha256: journal
                .backup
                .as_ref()
                .map(|backup| backup.manifest_sha256.clone()),
            first_failure: journal.first_failure.clone(),
            recovery_decision: journal.recovery_decision,
        }
    }
}

pub trait ChangedUpdateEffects {
    fn create_verified_backup(&mut self) -> Result<BackupClosure>;
    fn persist_journal(&mut self, journal: &DeploymentJournal) -> Result<()>;
    fn write_fence(&mut self, journal: &DeploymentJournal) -> Result<()>;
    fn stop_service(&mut self) -> Result<()>;
    fn stage_artifacts(&mut self) -> Result<()>;
    fn apply_migrations(&mut self) -> Result<Vec<MigrationIdentity>>;
    fn activate_target(&mut self) -> Result<()>;
    fn start_target_once(&mut self, journal: &DeploymentJournal) -> Result<()>;
    fn verify_target(&mut self) -> Result<()>;
    fn observe_migrations(&mut self) -> Result<Vec<MigrationIdentity>>;
    fn restore_source(&mut self, journal: &DeploymentJournal) -> Result<()>;
    fn verify_source(&mut self) -> Result<()>;
    fn clear_fence(&mut self) -> Result<()>;
}

pub fn execute_changed_update(
    mut journal: DeploymentJournal,
    effects: &mut impl ChangedUpdateEffects,
) -> Result<DeployReceipt> {
    if journal.phase != OperationPhase::Preflight {
        return Err(OpsError::message(
            "new changed update journal is not in preflight",
        ));
    }
    journal.validate()?;
    effects.persist_journal(&journal)?;

    let backup = match effects.create_verified_backup() {
        Ok(backup) => backup,
        Err(error) => {
            journal.record_failure(error.to_string());
            let _ = effects.persist_journal(&journal);
            return Err(error);
        }
    };
    journal.backup = Some(backup);
    transition_and_persist(&mut journal, OperationPhase::BackupVerified, effects)?;

    if let Err(error) = effects.write_fence(&journal) {
        journal.record_failure(error.to_string());
        let _ = effects.persist_journal(&journal);
        return Err(error);
    }
    if let Err(error) = transition_and_persist(&mut journal, OperationPhase::Fenced, effects) {
        return recover_after_failure(journal, effects, error);
    }

    let result = (|| {
        effects.stop_service()?;
        transition_and_persist(&mut journal, OperationPhase::ServiceStopped, effects)?;

        effects.stage_artifacts()?;
        transition_and_persist(&mut journal, OperationPhase::ArtifactsStaged, effects)?;

        journal.migration_after = Some(effects.apply_migrations()?);
        transition_and_persist(&mut journal, OperationPhase::MigrationClassified, effects)?;

        effects.activate_target()?;
        transition_and_persist(&mut journal, OperationPhase::Activated, effects)?;

        effects.start_target_once(&journal)?;
        transition_and_persist(&mut journal, OperationPhase::ServiceStarting, effects)?;

        effects.verify_target()?;
        transition_and_persist(&mut journal, OperationPhase::PostStartVerifying, effects)?;

        transition_and_persist(&mut journal, OperationPhase::Accepted, effects)
    })();

    match result {
        Ok(()) => {
            if let Err(error) = effects.clear_fence() {
                journal.record_failure(error.to_string());
                let _ = effects.persist_journal(&journal);
                return Err(OpsError::message(format!(
                    "target release was accepted but its matching deployment fence could not be cleared; retry deploy recover: {error}"
                )));
            }
            Ok(DeployReceipt::from_journal(
                &journal,
                DeployOutcome::Accepted,
            ))
        }
        Err(error) => recover_after_failure(journal, effects, error),
    }
}

pub fn recover_interrupted_update(
    mut journal: DeploymentJournal,
    effects: &mut impl ChangedUpdateEffects,
) -> Result<DeployReceipt> {
    journal.validate()?;
    match journal.phase {
        OperationPhase::Accepted => {
            effects.verify_target()?;
            effects.clear_fence()?;
            return Ok(DeployReceipt::from_journal(
                &journal,
                DeployOutcome::Accepted,
            ));
        }
        OperationPhase::RolledBack => {
            effects.verify_source()?;
            effects.clear_fence()?;
            return Ok(DeployReceipt::from_journal(
                &journal,
                DeployOutcome::RolledBack,
            ));
        }
        OperationPhase::Abandoned => {
            return DeployReceipt::abandoned(&journal);
        }
        OperationPhase::NoOp => {
            return Err(OpsError::message(
                "a no-op has no interrupted deployment state to recover",
            ));
        }
        _ => {}
    }
    if journal.backup.is_none() {
        return Err(OpsError::message(
            "interrupted update journal has no verified backup closure",
        ));
    }
    if let Err(error) = effects.stop_service() {
        journal.record_failure(error.to_string());
        journal.recovery_decision = Some(RecoveryDecision::RecoveryBlocked);
        force_terminal(&mut journal, OperationPhase::RecoveryBlocked, effects)?;
        return Err(OpsError::message(format!(
            "interrupted recovery could not stop the service: {error}"
        )));
    }
    let first = journal
        .first_failure
        .clone()
        .unwrap_or_else(|| "interrupted update requires recovery".to_string());
    recover_after_failure(journal, effects, OpsError::message(first))
}

fn recover_after_failure(
    mut journal: DeploymentJournal,
    effects: &mut impl ChangedUpdateEffects,
    error: OpsError,
) -> Result<DeployReceipt> {
    journal.record_failure(error.to_string());
    let observed = effects.observe_migrations().ok();
    let decision = classify_recovery(&journal.migration_before, observed.as_deref());
    journal.recovery_decision = Some(decision);
    match decision {
        RecoveryDecision::SafeBinaryRollback => {
            if let Err(rollback_error) = effects
                .restore_source(&journal)
                .and_then(|()| effects.verify_source())
            {
                journal.record_failure(rollback_error.to_string());
                journal.recovery_decision = Some(RecoveryDecision::RecoveryBlocked);
                force_terminal(&mut journal, OperationPhase::RecoveryBlocked, effects)?;
                return Err(OpsError::message(format!(
                    "update failed and safe rollback was blocked: {}; recovery failure: {rollback_error}",
                    journal
                        .first_failure
                        .as_deref()
                        .unwrap_or("unknown update failure")
                )));
            }
            force_terminal(&mut journal, OperationPhase::RolledBack, effects)?;
            effects.clear_fence()?;
            Err(OpsError::message(format!(
                "update failed; the exact prior release was restored and verified: {}",
                journal
                    .first_failure
                    .as_deref()
                    .unwrap_or("unknown update failure")
            )))
        }
        RecoveryDecision::DataAwareRestoreRequired => {
            force_terminal(&mut journal, OperationPhase::RestoreRequired, effects)?;
            Err(OpsError::message(format!(
                "update failed after the PostgreSQL migration ledger changed; service remains fenced and data-aware restore is required: {}",
                journal
                    .first_failure
                    .as_deref()
                    .unwrap_or("unknown update failure")
            )))
        }
        RecoveryDecision::RecoveryBlocked => {
            force_terminal(&mut journal, OperationPhase::RecoveryBlocked, effects)?;
            Err(OpsError::message(format!(
                "update failed and the PostgreSQL migration ledger is unreadable; service remains fenced: {}",
                journal
                    .first_failure
                    .as_deref()
                    .unwrap_or("unknown update failure")
            )))
        }
    }
}

fn transition_and_persist(
    journal: &mut DeploymentJournal,
    phase: OperationPhase,
    effects: &mut impl ChangedUpdateEffects,
) -> Result<()> {
    journal.transition(phase)?;
    effects.persist_journal(journal)
}

fn force_terminal(
    journal: &mut DeploymentJournal,
    phase: OperationPhase,
    effects: &mut impl ChangedUpdateEffects,
) -> Result<()> {
    if journal.phase != phase && journal.transition(phase).is_err() {
        journal.phase = phase;
    }
    effects.persist_journal(journal)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use uuid::Uuid;

    use super::*;

    #[derive(Default)]
    struct Effects {
        calls: Vec<&'static str>,
        fail_at: Option<&'static str>,
        observed: Option<Vec<MigrationIdentity>>,
        journals: Vec<DeploymentJournal>,
    }

    impl Effects {
        fn call(&mut self, name: &'static str) -> Result<()> {
            self.calls.push(name);
            if self.fail_at == Some(name) {
                Err(OpsError::message(format!("injected {name} failure")))
            } else {
                Ok(())
            }
        }
    }

    impl ChangedUpdateEffects for Effects {
        fn create_verified_backup(&mut self) -> Result<BackupClosure> {
            self.call("backup")?;
            Ok(backup())
        }
        fn persist_journal(&mut self, journal: &DeploymentJournal) -> Result<()> {
            self.call("journal")?;
            self.journals.push(journal.clone());
            Ok(())
        }
        fn write_fence(&mut self, _journal: &DeploymentJournal) -> Result<()> {
            self.call("fence")
        }
        fn stop_service(&mut self) -> Result<()> {
            self.call("stop")
        }
        fn stage_artifacts(&mut self) -> Result<()> {
            self.call("stage")
        }
        fn apply_migrations(&mut self) -> Result<Vec<MigrationIdentity>> {
            self.call("migrate")?;
            Ok(self.observed.clone().unwrap_or_else(marker))
        }
        fn activate_target(&mut self) -> Result<()> {
            self.call("activate")
        }
        fn start_target_once(&mut self, _journal: &DeploymentJournal) -> Result<()> {
            self.call("start")
        }
        fn verify_target(&mut self) -> Result<()> {
            self.call("verify-target")
        }
        fn observe_migrations(&mut self) -> Result<Vec<MigrationIdentity>> {
            self.call("observe")?;
            self.observed
                .clone()
                .ok_or_else(|| OpsError::message("migration ledger unavailable"))
        }
        fn restore_source(&mut self, _journal: &DeploymentJournal) -> Result<()> {
            self.call("restore-source")
        }
        fn verify_source(&mut self) -> Result<()> {
            self.call("verify-source")
        }
        fn clear_fence(&mut self) -> Result<()> {
            self.call("clear-fence")
        }
    }

    #[test]
    fn changed_update_orders_backup_fence_stop_migrate_start_and_acceptance() -> Result<()> {
        let mut effects = Effects {
            observed: Some(marker()),
            ..Effects::default()
        };
        let receipt = execute_changed_update(journal(), &mut effects)?;
        assert_eq!(receipt.outcome, DeployOutcome::Accepted);
        assert_eq!(receipt.phase, OperationPhase::Accepted);
        assert_eq!(effects.calls.first(), Some(&"journal"));
        assert_eq!(
            effects.journals.first().map(|value| value.phase),
            Some(OperationPhase::Preflight)
        );
        assert!(effects
            .journals
            .first()
            .is_some_and(|value| value.backup.is_none()));
        assert_order(
            &effects.calls,
            &[
                "backup",
                "fence",
                "stop",
                "stage",
                "migrate",
                "activate",
                "start",
                "verify-target",
                "clear-fence",
            ],
        )?;
        Ok(())
    }

    #[test]
    fn exact_noop_receipt_performs_no_effect() -> Result<()> {
        let receipt = DeployReceipt::no_op(&journal())?;
        assert_eq!(receipt.outcome, DeployOutcome::NoOp);
        assert_eq!(receipt.phase, OperationPhase::Accepted);
        assert!(receipt.backup_manifest_sha256.is_none());
        Ok(())
    }

    #[test]
    fn backup_failure_remains_durably_preflight_and_records_first_cause() -> Result<()> {
        let mut effects = Effects {
            fail_at: Some("backup"),
            ..Effects::default()
        };
        let Err(error) = execute_changed_update(journal(), &mut effects) else {
            return Err(OpsError::message(
                "injected backup failure unexpectedly passed",
            ));
        };
        assert!(error.to_string().contains("injected backup failure"));
        let durable = effects
            .journals
            .last()
            .ok_or_else(|| OpsError::message("backup failure did not persist its journal"))?;
        assert_eq!(durable.phase, OperationPhase::Preflight);
        assert!(durable.backup.is_none());
        assert_eq!(
            durable.first_failure.as_deref(),
            Some("injected backup failure")
        );
        Ok(())
    }

    #[test]
    fn abandoned_retry_has_no_external_effect() -> Result<()> {
        let mut abandoned = journal();
        abandoned.transition(OperationPhase::Abandoned)?;
        let mut effects = Effects::default();
        let receipt = recover_interrupted_update(abandoned, &mut effects)?;
        assert_eq!(receipt.outcome, DeployOutcome::Abandoned);
        assert_eq!(receipt.phase, OperationPhase::Abandoned);
        assert!(effects.calls.is_empty());
        Ok(())
    }

    #[test]
    fn pre_ledger_failure_restores_and_verifies_source_before_clearing_fence() -> Result<()> {
        let mut effects = Effects {
            fail_at: Some("stage"),
            observed: Some(marker()),
            ..Effects::default()
        };
        let Err(error) = execute_changed_update(journal(), &mut effects) else {
            return Err(OpsError::message("injected update unexpectedly passed"));
        };
        assert!(error.to_string().contains("prior release was restored"));
        assert_order(
            &effects.calls,
            &[
                "stage",
                "observe",
                "restore-source",
                "verify-source",
                "clear-fence",
            ],
        )?;
        assert_eq!(
            effects.journals.last().map(|value| value.phase),
            Some(OperationPhase::RolledBack)
        );
        Ok(())
    }

    #[test]
    fn changed_ledger_never_attempts_binary_rollback() -> Result<()> {
        let mut changed = marker();
        changed.push(MigrationIdentity {
            version: 55,
            name: "future".to_string(),
            checksum: "5".repeat(64),
        });
        let mut effects = Effects {
            fail_at: Some("verify-target"),
            observed: Some(changed),
            ..Effects::default()
        };
        let Err(error) = execute_changed_update(journal(), &mut effects) else {
            return Err(OpsError::message("injected update unexpectedly passed"));
        };
        assert!(error.to_string().contains("data-aware restore is required"));
        assert!(!effects.calls.contains(&"restore-source"));
        assert!(!effects.calls.contains(&"clear-fence"));
        assert_eq!(
            effects.journals.last().map(|value| value.phase),
            Some(OperationPhase::RestoreRequired)
        );
        Ok(())
    }

    #[test]
    fn unreadable_ledger_remains_recovery_blocked() -> Result<()> {
        let mut effects = Effects {
            fail_at: Some("verify-target"),
            observed: None,
            ..Effects::default()
        };
        let Err(error) = execute_changed_update(journal(), &mut effects) else {
            return Err(OpsError::message("injected update unexpectedly passed"));
        };
        assert!(error.to_string().contains("ledger is unreadable"));
        assert_eq!(
            effects.journals.last().map(|value| value.phase),
            Some(OperationPhase::RecoveryBlocked)
        );
        Ok(())
    }

    #[test]
    fn accepted_target_with_uncleared_fence_is_recoverable_without_rollback() -> Result<()> {
        let mut effects = Effects {
            fail_at: Some("clear-fence"),
            observed: Some(marker()),
            ..Effects::default()
        };
        let Err(error) = execute_changed_update(journal(), &mut effects) else {
            return Err(OpsError::message(
                "fence cleanup failure unexpectedly passed",
            ));
        };
        assert!(error.to_string().contains("target release was accepted"));
        assert!(!effects.calls.contains(&"restore-source"));
        assert!(!effects.calls.contains(&"observe"));
        assert_eq!(
            effects.journals.last().map(|value| value.phase),
            Some(OperationPhase::Accepted)
        );
        Ok(())
    }

    #[test]
    fn accepted_retry_only_reverifies_target_and_clears_matching_fence() -> Result<()> {
        let mut accepted = journal();
        accepted.backup = Some(backup());
        accepted.phase = OperationPhase::Accepted;
        let mut effects = Effects::default();
        let receipt = recover_interrupted_update(accepted, &mut effects)?;
        assert_eq!(receipt.outcome, DeployOutcome::Accepted);
        assert_eq!(effects.calls, ["verify-target", "clear-fence"]);
        Ok(())
    }

    #[test]
    fn rolled_back_retry_only_reverifies_source_and_clears_matching_fence() -> Result<()> {
        let mut rolled_back = journal();
        rolled_back.backup = Some(backup());
        rolled_back.phase = OperationPhase::RolledBack;
        let mut effects = Effects::default();
        let receipt = recover_interrupted_update(rolled_back, &mut effects)?;
        assert_eq!(receipt.outcome, DeployOutcome::RolledBack);
        assert_eq!(effects.calls, ["verify-source", "clear-fence"]);
        Ok(())
    }

    fn assert_order(haystack: &[&'static str], needles: &[&'static str]) -> Result<()> {
        let mut prior = None;
        for needle in needles {
            let index = haystack
                .iter()
                .position(|value| value == needle)
                .ok_or_else(|| OpsError::message(format!("missing call {needle}: {haystack:?}")))?;
            assert!(prior.is_none_or(|value| index > value));
            prior = Some(index);
        }
        Ok(())
    }

    fn marker() -> Vec<MigrationIdentity> {
        vec![MigrationIdentity {
            version: 54,
            name: "align-instance-kind-and-desired-state".to_string(),
            checksum: "4".repeat(64),
        }]
    }

    fn backup() -> BackupClosure {
        BackupClosure {
            dump: PathBuf::from("/var/backups/lkjmc/op/database.dump"),
            manifest: PathBuf::from("/var/backups/lkjmc/op/database.manifest"),
            metadata: PathBuf::from("/var/backups/lkjmc/op/metadata.json"),
            checksums: PathBuf::from("/var/backups/lkjmc/op/checksums.sha256"),
            dump_sha256: "d".repeat(64),
            manifest_sha256: "e".repeat(64),
            metadata_sha256: "f".repeat(64),
            source_commit: "a".repeat(40),
            schema_identity: "1".repeat(64),
            migration_identity: "2".repeat(64),
        }
    }

    fn journal() -> DeploymentJournal {
        DeploymentJournal {
            schema_version: 1,
            operation_id: Uuid::new_v4(),
            source_commit: "a".repeat(40),
            source_manifest_sha256: "9".repeat(64),
            target_commit: "b".repeat(40),
            manifest_sha256: "c".repeat(64),
            state_directory: PathBuf::from(format!(
                "/var/lib/private/lkjmc-deployments/{}",
                Uuid::new_v4()
            )),
            prior_release_root: PathBuf::from(format!("/opt/lkjmc/releases/{}", "a".repeat(40))),
            prior_unit_sha256: "d".repeat(64),
            prior_fence_dropin_sha256: "1".repeat(64),
            prior_plugins: BTreeMap::from([
                ("alpha-world".to_string(), "e".repeat(64)),
                ("front-door".to_string(), "f".repeat(64)),
            ]),
            migration_before: marker(),
            migration_after: None,
            backup_path: PathBuf::from("/var/backups/lkjmc/op"),
            backup: None,
            rollback_snapshot: Some("snapshot-a".to_string()),
            phase: OperationPhase::Preflight,
            first_failure: None,
            recovery_decision: None,
        }
    }
}
