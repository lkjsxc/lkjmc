use postgres::{GenericClient, Row};
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferIntentRecord {
    pub id: Uuid,
    pub temporary_instance_id: String,
    pub player_uuid: Uuid,
    pub player_name: String,
    pub state: String,
}

pub struct NewTransferIntent<'a> {
    pub id: Uuid,
    pub temporary_instance_id: &'a str,
    pub player_uuid: Uuid,
    pub player_name: &'a str,
    pub expires_in_seconds: i32,
    pub metadata: Value,
}

pub fn create_intent(
    client: &mut impl GenericClient,
    intent: NewTransferIntent<'_>,
) -> Result<TransferIntentRecord, StoreError> {
    let row = client.query_one(
        "insert into temporary_transfer_intents
         (id, temporary_instance_id, player_uuid, player_name, state,
          expires_at, metadata)
         values ($1, $2, $3, $4, 'queued',
          now() + ($5::integer * interval '1 second'), $6)
         returning id, temporary_instance_id, player_uuid, player_name, state",
        &[
            &intent.id,
            &intent.temporary_instance_id,
            &intent.player_uuid,
            &intent.player_name,
            &intent.expires_in_seconds,
            &intent.metadata,
        ],
    )?;
    Ok(intent_from_row(&row))
}

pub fn get_intent(
    client: &mut impl GenericClient,
    id: Uuid,
) -> Result<Option<TransferIntentRecord>, StoreError> {
    let row = client.query_opt(
        "select id, temporary_instance_id, player_uuid, player_name, state
         from temporary_transfer_intents where id = $1",
        &[&id],
    )?;
    Ok(row.map(|row| intent_from_row(&row)))
}

fn intent_from_row(row: &Row) -> TransferIntentRecord {
    TransferIntentRecord {
        id: row.get(0),
        temporary_instance_id: row.get(1),
        player_uuid: row.get(2),
        player_name: row.get(3),
        state: row.get(4),
    }
}
