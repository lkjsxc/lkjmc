use postgres::{Client, Row};
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapRunRecord {
    pub id: Uuid,
    pub profile: String,
    pub requested_by: String,
    pub result: String,
    pub diagnostics: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapStepRecord {
    pub id: Uuid,
    pub run_id: Uuid,
    pub step_order: i32,
    pub effect_kind: String,
    pub target: String,
    pub result: String,
    pub diagnostic: Option<String>,
}

pub struct NewBootstrapRun<'a> {
    pub id: Uuid,
    pub profile: &'a str,
    pub requested_by: &'a str,
    pub result: &'a str,
    pub diagnostics: Value,
}

pub struct NewBootstrapStep<'a> {
    pub id: Uuid,
    pub run_id: Uuid,
    pub step_order: i32,
    pub effect_kind: &'a str,
    pub target: &'a str,
    pub result: &'a str,
    pub diagnostic: Option<&'a str>,
}

pub fn try_apply_lock(client: &mut Client) -> Result<bool, StoreError> {
    Ok(client
        .query_one(
            "select pg_try_advisory_lock(hashtext('lkjmc-bootstrap-apply'))",
            &[],
        )?
        .get(0))
}

pub fn release_apply_lock(client: &mut Client) -> Result<(), StoreError> {
    client.execute(
        "select pg_advisory_unlock(hashtext('lkjmc-bootstrap-apply'))",
        &[],
    )?;
    Ok(())
}

pub fn fail_unfinished_runs(client: &mut Client) -> Result<u64, StoreError> {
    Ok(client.execute(
        "update bootstrap_runs set result = 'failed', finished_at = now()
         where result = 'running' and finished_at is null",
        &[],
    )?)
}

pub fn create_run(client: &mut Client, run: NewBootstrapRun<'_>) -> Result<(), StoreError> {
    client.execute(
        "insert into bootstrap_runs
         (id, profile, requested_by, result, diagnostics)
         values ($1, $2, $3, $4, $5)",
        &[
            &run.id,
            &run.profile,
            &run.requested_by,
            &run.result,
            &run.diagnostics,
        ],
    )?;
    Ok(())
}

pub fn finish_run(
    client: &mut Client,
    id: Uuid,
    result: &str,
    diagnostics: Value,
) -> Result<(), StoreError> {
    client.execute(
        "update bootstrap_runs set result = $2, diagnostics = $3, finished_at = now()
         where id = $1",
        &[&id, &result, &diagnostics],
    )?;
    Ok(())
}

pub fn get_run(client: &mut Client, id: Uuid) -> Result<Option<BootstrapRunRecord>, StoreError> {
    let row = client.query_opt(
        "select id, profile, requested_by, result, diagnostics
         from bootstrap_runs where id = $1",
        &[&id],
    )?;
    Ok(row.map(run_from_row))
}

pub fn insert_step(client: &mut Client, step: NewBootstrapStep<'_>) -> Result<(), StoreError> {
    client.execute(
        "insert into bootstrap_steps
         (id, run_id, step_order, effect_kind, target, result, diagnostic)
         values ($1, $2, $3, $4, $5, $6, $7)",
        &[
            &step.id,
            &step.run_id,
            &step.step_order,
            &step.effect_kind,
            &step.target,
            &step.result,
            &step.diagnostic,
        ],
    )?;
    Ok(())
}

pub fn steps_for_run(
    client: &mut Client,
    run_id: Uuid,
) -> Result<Vec<BootstrapStepRecord>, StoreError> {
    let rows = client.query(
        "select id, run_id, step_order, effect_kind, target, result, diagnostic
         from bootstrap_steps where run_id = $1 order by step_order",
        &[&run_id],
    )?;
    Ok(rows.into_iter().map(step_from_row).collect())
}

fn run_from_row(row: Row) -> BootstrapRunRecord {
    BootstrapRunRecord {
        id: row.get(0),
        profile: row.get(1),
        requested_by: row.get(2),
        result: row.get(3),
        diagnostics: row.get(4),
    }
}

fn step_from_row(row: Row) -> BootstrapStepRecord {
    BootstrapStepRecord {
        id: row.get(0),
        run_id: row.get(1),
        step_order: row.get(2),
        effect_kind: row.get(3),
        target: row.get(4),
        result: row.get(5),
        diagnostic: row.get(6),
    }
}
