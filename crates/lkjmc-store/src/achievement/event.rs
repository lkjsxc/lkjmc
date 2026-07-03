use postgres::{Client, GenericClient};
use uuid::Uuid;

use crate::error::StoreError;
use crate::player;

use super::seed_defaults;
use super::support::progress_definition;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AchievementEventOutcome {
    pub claimed: Vec<String>,
    pub missing_definitions: Vec<String>,
}

pub fn apply_event(
    client: &mut Client,
    player_uuid: Uuid,
    criteria_kind: &str,
    amount: i64,
    correlation_id: Option<Uuid>,
) -> Result<Vec<String>, StoreError> {
    Ok(apply_event_for_player(
        client,
        player_uuid,
        None,
        criteria_kind,
        amount,
        correlation_id,
    )?
    .claimed)
}

pub fn apply_event_for_player(
    client: &mut impl GenericClient,
    player_uuid: Uuid,
    player_name: Option<&str>,
    criteria_kind: &str,
    amount: i64,
    correlation_id: Option<Uuid>,
) -> Result<AchievementEventOutcome, StoreError> {
    seed_defaults(client)?;
    player::ensure_identity(client, player_uuid, player_name)?;
    let mut outcome = AchievementEventOutcome::default();
    for definition in lkjmc_core::achievement::by_criteria(criteria_kind) {
        if !definition_exists(client, definition.id)? {
            outcome.missing_definitions.push(definition.id.to_string());
            continue;
        }
        if progress_definition(client, player_uuid, definition, amount, correlation_id)? {
            outcome.claimed.push(definition.id.to_string());
        }
    }
    Ok(outcome)
}

fn definition_exists(client: &mut impl GenericClient, id: &str) -> Result<bool, StoreError> {
    Ok(client
        .query_opt("select id from achievements where id = $1", &[&id])?
        .is_some())
}
