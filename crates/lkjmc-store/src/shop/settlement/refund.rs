use postgres::Client;
use uuid::Uuid;

use crate::error::StoreError;

use super::lock;

pub fn refund_purchase(
    client: &mut Client,
    player: Uuid,
    correlation: Uuid,
    reason: &str,
) -> Result<bool, StoreError> {
    let mut tx = client.transaction()?;
    lock(&mut tx, correlation)?;
    if tx
        .query_opt(
            "select 1 from shop_purchases where player_uuid = $1 and correlation_id = $2",
            &[&player, &correlation],
        )?
        .is_none()
    {
        tx.commit()?;
        return Ok(false);
    }
    let debit = tx.query_opt(
        "select -delta from points_ledger where player_uuid = $1 and correlation_id = $2
         and reason = 'shop.purchase' and delta < 0",
        &[&player, &correlation],
    )?;
    let Some(debit) = debit else {
        tx.commit()?;
        return Ok(false);
    };
    fail_pending_delivery(&mut tx, correlation)?;
    let refund = Uuid::new_v5(&correlation, b"shop-purchase-refund");
    if tx
        .query_opt(
            "select 1 from points_ledger where correlation_id = $1",
            &[&refund],
        )?
        .is_some()
    {
        tx.commit()?;
        return Ok(false);
    }
    crate::points::grant_with_correlation(&mut tx, player, debit.get(0), reason, Some(refund))?;
    tx.commit()?;
    Ok(true)
}

fn fail_pending_delivery(
    tx: &mut postgres::Transaction<'_>,
    correlation: Uuid,
) -> Result<(), StoreError> {
    let workflow = tx.query_opt(
        "select id, state, revision, fence from item_delivery_workflows
         where correlation_id = $1 for update",
        &[&correlation],
    )?;
    let Some(workflow) = workflow else {
        return Ok(());
    };
    let state: String = workflow.get(1);
    match state.as_str() {
        "pending_receipt" => {
            crate::data_workflows::fail_in_transaction(
                tx,
                crate::data_workflows::WorkflowTable::Delivery,
                workflow.get(0),
                workflow.get(2),
                workflow.get(3),
                "shop purchase refunded",
            )?;
            Ok(())
        }
        "failed" => Ok(()),
        _ => Err(StoreError::invalid_state(
            "received delivery cannot be refunded",
        )),
    }
}
