use postgres::GenericClient;
use uuid::Uuid;

use crate::error::StoreError;

pub fn refund_session(
    client: &mut impl GenericClient,
    session_id: Uuid,
    ledger_reason: &str,
    failure_reason: &str,
) -> Result<Option<Uuid>, StoreError> {
    let row = client.query_opt(
        "select buyer_uuid, points_cost, refund_ledger_id, state
         from adventure_sessions where id = $1 for update",
        &[&session_id],
    )?;
    let Some(row) = row else {
        return Ok(None);
    };
    let existing: Option<Uuid> = row.get(2);
    if existing.is_some() {
        return Ok(existing);
    }
    let state: String = row.get(3);
    if !matches!(
        state.as_str(),
        "pending" | "starting" | "ready" | "cancelled"
    ) {
        return Ok(None);
    }
    let buyer: Uuid = row.get(0);
    let cost: i64 = row.get(1);
    let correlation = Uuid::new_v5(&session_id, b"adventure-session-refund");
    let ledger = crate::points::grant_with_correlation(
        client,
        buyer,
        cost,
        ledger_reason,
        Some(correlation),
    )?;
    client.execute(
        "update adventure_sessions set state = 'refunded', failure_reason = $2,
         refund_ledger_id = $3, updated_at = now() where id = $1",
        &[&session_id, &failure_reason, &ledger],
    )?;
    Ok(Some(ledger))
}

#[cfg(test)]
pub fn refund_correlation(session_id: Uuid) -> Uuid {
    Uuid::new_v5(&session_id, b"adventure-session-refund")
}

#[cfg(test)]
mod tests {
    use super::refund_correlation;
    use uuid::Uuid;

    #[test]
    fn temp_refund_consistent() {
        let session = Uuid::new_v4();
        assert_ne!(refund_correlation(session), session);
        assert_eq!(refund_correlation(session), refund_correlation(session));
    }
}
