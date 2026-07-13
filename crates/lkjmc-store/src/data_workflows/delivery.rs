use postgres::{Client, GenericClient};
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

use super::{change_feed, record, WorkflowRecord};

pub struct NewDelivery {
    pub id: Uuid,
    pub purchase_id: Uuid,
    pub player_uuid: Uuid,
    pub delivery: Value,
    pub correlation_id: Uuid,
}

pub fn create_delivery(
    client: &mut Client,
    new: NewDelivery,
) -> Result<WorkflowRecord, StoreError> {
    let mut tx = client.transaction()?;
    let result = insert_delivery(&mut tx, &new)?;
    tx.commit()?;
    Ok(result)
}

pub(crate) fn insert_delivery(
    client: &mut impl GenericClient,
    new: &NewDelivery,
) -> Result<WorkflowRecord, StoreError> {
    if !new.delivery.is_object() {
        return Err(StoreError::invalid_state(
            "delivery intent must be an object",
        ));
    }
    if let Some(row) = client.query_opt(
        "select id, state, revision, fence, correlation_id, purchase_id,
         player_uuid, delivery from item_delivery_workflows
         where correlation_id = $1 for update",
        &[&new.correlation_id],
    )? {
        let same = row.get::<_, Uuid>(5) == new.purchase_id
            && row.get::<_, Uuid>(6) == new.player_uuid
            && row.get::<_, Value>(7) == new.delivery;
        if !same {
            return Err(StoreError::invalid_state("changed delivery replay"));
        }
        return record(&row, true);
    }
    let purchase = client
        .query_opt(
            "select player_uuid from shop_purchases where id = $1 for update",
            &[&new.purchase_id],
        )?
        .ok_or_else(|| StoreError::invalid_state("delivery purchase missing"))?;
    if purchase.get::<_, Uuid>(0) != new.player_uuid {
        return Err(StoreError::invalid_state(
            "delivery purchase owner mismatch",
        ));
    }
    let row = client.query_one(
        "insert into item_delivery_workflows
         (id, purchase_id, player_uuid, delivery, state, revision, fence, correlation_id)
         values ($1,$2,$3,$4,'pending_receipt',1,1,$5)
         returning id, state, revision, fence, correlation_id",
        &[
            &new.id,
            &new.purchase_id,
            &new.player_uuid,
            &new.delivery,
            &new.correlation_id,
        ],
    )?;
    change_feed::append(
        client,
        "delivery",
        new.id,
        1,
        new.correlation_id,
        "pending_receipt",
    )?;
    record(&row, false)
}
