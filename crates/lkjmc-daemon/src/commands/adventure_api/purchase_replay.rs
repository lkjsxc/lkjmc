use serde_json::{json, Value};
use uuid::Uuid;

use crate::support::instance_helpers::store;

pub(super) fn by_correlation(
    client: &mut postgres::Client,
    player: Uuid,
    correlation: Uuid,
) -> Result<Option<Value>, String> {
    let session = store(lkjmc_store::temporary::get_session(client, correlation))?;
    let Some(session) = session else {
        return Ok(None);
    };
    if session.buyer_uuid != player {
        return Err("adventure correlation belongs to another player".to_string());
    }
    Ok(Some(json!({
        "sessionId": session.id.to_string(),
        "adventureId": session.adventure_kind,
        "temporaryInstanceId": session.temporary_instance_id,
        "targetServer": session.temporary_instance_id,
        "pricePoints": session.points_cost,
        "state": session.state,
        "duplicate": true,
        "deliveryStatus": "settled-replay"
    })))
}
