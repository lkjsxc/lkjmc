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
    Ok(Some(response(&session)))
}

fn response(session: &lkjmc_store::temporary::AdventureSessionRecord) -> Value {
    json!({
        "sessionId": session.id.to_string(),
        "adventureId": session.adventure_kind,
        "temporaryInstanceId": session.temporary_instance_id,
        "targetServer": session.temporary_instance_id,
        "pricePoints": session.points_cost,
        "state": session.state,
        "duplicate": true,
        "deliveryStatus": "settled-replay"
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_uses_settled_adventure_facts_after_catalog_mutation() {
        let session = lkjmc_store::temporary::AdventureSessionRecord {
            id: Uuid::nil(),
            adventure_kind: "end-expedition".to_string(),
            buyer_uuid: Uuid::nil(),
            temporary_instance_id: "end-000000000000".to_string(),
            points_cost: 20,
            state: "ready".to_string(),
        };
        let replay = response(&session);
        assert_eq!(replay["adventureId"], "end-expedition");
        assert_eq!(replay["pricePoints"], 20);
        assert_eq!(replay["targetServer"], "end-000000000000");
    }
}
