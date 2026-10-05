use super::{game, receipt, uuid};
use crate::{
    auth::hash,
    error::{Error, Result},
};
use serde_json::{Value, json};
use sqlx::PgConnection;
use uuid::Uuid;
mod identity;

pub(super) async fn success(
    db: &mut PgConnection,
    id: Uuid,
    actor: Uuid,
    server: Option<Uuid>,
    kind: &str,
    payload: &Value,
    result: &Value,
) -> Result<()> {
    match kind {
        "server.create" | "server.start" | "server.stop" | "server.restore" => {
            receipt(result)?;
            let expected = if kind == "server.start" {
                "running"
            } else {
                "stopped"
            };
            if result.get("observed").and_then(Value::as_str) != Some(expected) {
                return Err(Error::invalid(
                    "text.the_actual_server_state_does_not_match_the_request",
                ));
            }
            sqlx::query("UPDATE servers SET observed=$2,last_observed_at=now(),error=NULL,address=coalesce($3,address),capabilities=capabilities || coalesce($4,'{}'::jsonb),empty_since=CASE WHEN $2='running' THEN now() ELSE NULL END WHERE id=$1")
                .bind(server).bind(expected).bind(result.get("address").and_then(Value::as_str)).bind(result.get("capabilities")).execute(&mut *db).await?;
            if kind == "server.restore" {
                sqlx::query("UPDATE backups SET state='ready' WHERE id=$1")
                    .bind(uuid(payload, "backup_id")?)
                    .execute(&mut *db)
                    .await?;
            }
        }
        "server.install" => {
            receipt(result)?;
            let expected: String = sqlx::query_scalar("SELECT sha256 FROM artifacts WHERE id=$1")
                .bind(uuid(payload, "artifact_id")?)
                .fetch_one(&mut *db)
                .await?;
            if result.get("sha256").and_then(Value::as_str) != Some(expected.as_str()) {
                return Err(Error::invalid("text.the_deployed_file_hashes_do_not_match"));
            }
        }
        "server.backup" | "official.backup" => {
            receipt(result)?;
            if result.get("verified").and_then(Value::as_bool) != Some(true) {
                return Err(Error::invalid("text.backup_verification_is_missing"));
            }
            if kind == "official.backup" {
                let step:Value=sqlx::query_scalar("SELECT to_jsonb(b) FROM official_backup_steps b WHERE job_id=$1 AND phase='released'").bind(id).fetch_optional(&mut *db).await?.ok_or_else(||Error::conflict("text.the_official_backup_and_service_resumption_have_not_finished"))?;
                if result["database"] != step["database_manifest"]
                    || result["world"] != step["world_manifest"]
                {
                    return Err(Error::conflict(
                        "text.the_database_and_worlds_cannot_be_verified_as_one_consi_4a979c0768",
                    ));
                }
            }
            sqlx::query("UPDATE backups SET state='ready',manifest=$2,error=NULL,completed_at=now() WHERE id=$1")
                .bind(uuid(payload, "backup_id")?)
                .bind(result)
                .execute(&mut *db)
                .await?;
        }
        "official.backup.prune" => {
            receipt(result)?;
            let backup = uuid(payload, "backup_id")?;
            if result["backup_id"] != json!(backup)
                || result["server_id"] != json!(server)
                || result["host_deleted"] != true
                || result["database_deleted"] != true
            {
                return Err(Error::invalid(
                    "text.the_pruning_deletion_record_does_not_match",
                ));
            }
            let changed=sqlx::query("UPDATE backups SET state='pruned',pruned_at=now(),error=NULL WHERE id=$1 AND prune_job_id=$2 AND state='pruning' AND NOT pinned AND database_pruned_at IS NOT NULL")
                .bind(backup).bind(id).execute(&mut *db).await?.rows_affected();
            if changed != 1 {
                return Err(Error::conflict(
                    "text.database_and_host_pruning_have_not_finished",
                ));
            }
        }
        "claim.sync" => {
            receipt(result)?;
            sqlx::query("UPDATE claims SET state='active' WHERE id=$1 AND job_id=$2 AND state IN ('pending','transferring')").bind(uuid(payload,"claim_id")?).bind(id).execute(&mut *db).await?;
            sqlx::query("UPDATE assets SET locked_claim_id=NULL WHERE locked_claim_id=$1 AND kind='land' AND state='placed'").bind(uuid(payload,"claim_id")?).execute(&mut *db).await?;
            // This event awards the personal first-claim achievement; team
            // contributions are recorded by the authenticated event endpoint.
            game::reward_event(db, actor, id, "claim.created", 1, None).await?;
        }
        "claim.release" => {
            receipt(result)?;
            sqlx::query("UPDATE claims SET state='released' WHERE id=$1 AND job_id=$2 AND state='releasing'").bind(uuid(payload,"claim_id")?).bind(id).execute(&mut *db).await?;
        }
        "home.set" => {
            receipt(result)?;
            let location = result
                .get("location")
                .ok_or_else(|| Error::invalid("text.the_home_position_is_missing"))?;
            if location.get("world_id").and_then(Value::as_str).is_none() {
                return Err(Error::invalid("text.the_home_world_is_missing"));
            }
            let profile = uuid(payload, "profile_id")?;
            sqlx::query("SELECT id FROM profiles WHERE id=$1 FOR UPDATE")
                .bind(profile)
                .fetch_one(&mut *db)
                .await?;
            let name = payload["name"]
                .as_str()
                .ok_or_else(|| Error::invalid("text.the_home_name_is_missing"))?;
            let count: i64 =
                sqlx::query_scalar("SELECT count(*) FROM homes WHERE profile_id=$1 AND name<>$2")
                    .bind(profile)
                    .bind(name)
                    .fetch_one(&mut *db)
                    .await?;
            if count >= 3 {
                return Err(Error::conflict("text.you_have_reached_your_home_limit"));
            }
            sqlx::query("INSERT INTO homes(id,profile_id,name,location) VALUES($1,$2,$3,$4) ON CONFLICT(profile_id,name) DO UPDATE SET location=$4").bind(Uuid::new_v4()).bind(profile).bind(name).bind(location).execute(&mut *db).await?;
        }
        "asset.capture" => {
            receipt(result)?;
            let asset = uuid(payload, "asset_id")?;
            let manifest = result
                .get("manifest")
                .filter(|m| m.is_object())
                .ok_or_else(|| Error::invalid("text.the_asset_save_information_is_missing"))?;
            if uuid(manifest, "asset_id")? != asset {
                return Err(Error::invalid("text.the_asset_id_does_not_match"));
            }
            if result.get("original_removed").and_then(Value::as_bool) != Some(true)
                && payload["kind"] != "land"
            {
                return Err(Error::invalid(
                    "text.removal_of_the_original_has_not_been_confirmed",
                ));
            }
            let digest = hash(&serde_json::to_string(manifest).map_err(Error::internal)?);
            for owner in manifest
                .get("required_consents")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
            {
                let owner = owner
                    .as_str()
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .ok_or_else(|| Error::invalid("text.the_pet_owner_id_is_invalid"))?;
                let consent:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM asset_consents WHERE asset_id=$1 AND owner=$2 AND manifest_sha256=$3)").bind(asset).bind(owner).bind(&digest).fetch_one(&mut *db).await?;
                if !consent {
                    return Err(Error::conflict(
                        "text.some_pet_owners_have_not_agreed_to_the_transfer",
                    ));
                }
            }
            sqlx::query("UPDATE assets SET state='escrowed',manifest=$2,manifest_sha256=$3,locked_claim_id=CASE WHEN kind='land' THEN claim_id ELSE NULL END WHERE id=$1 AND job_id=$4 AND state='capturing'").bind(asset).bind(manifest).bind(digest).bind(id).execute(&mut *db).await?;
        }
        "asset.place" | "asset.receive" => {
            receipt(result)?;
            let asset = uuid(payload, "asset_id")?;
            if uuid(result, "asset_id")? != asset {
                return Err(Error::invalid("text.the_delivered_asset_id_does_not_match"));
            }
            sqlx::query("UPDATE assets SET state=$2,locked_claim_id=NULL WHERE id=$1 AND job_id=$3 AND state='placing'")
                .bind(asset)
                .bind(if kind == "asset.place" {
                    "placed"
                } else {
                    "delivered"
                })
                .bind(id)
                .execute(&mut *db)
                .await?;
        }
        "npc.sell" => {
            receipt(result)?;
            if result.get("removed").and_then(Value::as_i64) != payload["amount"].as_i64() {
                return Err(Error::invalid(
                    "text.the_number_of_collected_materials_does_not_match",
                ));
            }
            let amount = payload["coins"]
                .as_i64()
                .ok_or_else(|| Error::invalid("text.the_buyback_amount_is_invalid"))?;
            crate::economy::book(
                db,
                actor,
                &format!("npc:{id}"),
                "npc",
                payload.clone(),
                &[(actor, amount)],
                true,
            )
            .await?;
        }
        "adventure.prepare" => {
            receipt(result)?;
            let adventure = uuid(payload, "adventure_id")?;
            let state: String =
                sqlx::query_scalar("SELECT state FROM adventures WHERE id=$1 FOR UPDATE")
                    .bind(adventure)
                    .fetch_one(&mut *db)
                    .await?;
            if !matches!(state.as_str(), "preparing" | "activating") {
                return Err(Error::conflict(
                    "text.adventure_preparation_has_changed_restore_its_original_f091e3ffc0",
                ));
            }
            if result.get("world_ready").and_then(Value::as_bool) != Some(true)
                || result.get("eyes_removed").and_then(Value::as_i64) != Some(12)
            {
                return Err(Error::invalid(
                    "text.world_generation_and_reserved_materials_could_not_be_verified",
                ));
            }
            let world_id = Uuid::new_v4();
            let native = uuid(result, "native_world_id")?;
            let name = result["world_name"]
                .as_str()
                .ok_or_else(|| Error::invalid("text.the_world_name_is_missing"))?;
            if name != format!("adventure_{adventure}") {
                return Err(Error::invalid(
                    "text.the_adventure_world_name_does_not_match",
                ));
            }
            sqlx::query("INSERT INTO worlds(id,server_id,name,kind,native_uuid,lifetime,environment,access_policy) VALUES($1,$2,$3,'private_end',$4,'temporary','end','participants')").bind(world_id).bind(server).bind(name).bind(native).execute(&mut *db).await?;
            sqlx::query("UPDATE wallets SET reserved=reserved-1000 WHERE owner=$1")
                .bind(actor)
                .execute(&mut *db)
                .await?;
            crate::economy::book(
                db,
                actor,
                &format!("adventure:{adventure}"),
                "adventure",
                json!({"adventure_id":adventure}),
                &[(actor, -1000), (crate::economy::TREASURY, 1000)],
                false,
            )
            .await?;
            sqlx::query("UPDATE adventures SET state='active',world_id=$2,opens_at=now(),expires_at=now()+interval '3 hours' WHERE id=$1").bind(adventure).bind(world_id).execute(&mut *db).await?;
        }
        "adventure.cancel" => {
            receipt(result)?;
            let adventure = uuid(payload, "adventure_id")?;
            if result.get("materials_returned").and_then(Value::as_bool) != Some(true) {
                return Err(Error::invalid("text.item_return_has_not_been_confirmed"));
            }
            let removed = result["eyes_removed"]
                .as_i64()
                .ok_or_else(|| Error::invalid("text.the_reserved_item_count_is_missing"))?;
            if !matches!(removed, 0 | 12) {
                return Err(Error::invalid("text.the_returned_item_count_is_invalid"));
            }
            let changed = sqlx::query(
                "UPDATE adventures SET state='refunded' WHERE id=$1 AND state='refunding'",
            )
            .bind(adventure)
            .execute(&mut *db)
            .await?
            .rows_affected();
            if changed > 0 {
                sqlx::query("UPDATE wallets SET reserved=reserved-1000 WHERE owner=$1")
                    .bind(actor)
                    .execute(&mut *db)
                    .await?;
                if removed == 12 {
                    let mut manifest = result.get("refund_manifest").cloned().ok_or_else(|| {
                        Error::invalid("text.the_return_item_save_information_is_missing")
                    })?;
                    if !manifest["items"].is_string() {
                        return Err(Error::invalid(
                            "text.the_return_item_save_information_is_invalid",
                        ));
                    }
                    let asset = Uuid::new_v4();
                    manifest["asset_id"] = json!(asset);
                    let digest = hash(&serde_json::to_string(&manifest).map_err(Error::internal)?);
                    sqlx::query("INSERT INTO assets(id,owner,kind,title,title_message,state,manifest,manifest_sha256,job_id) VALUES($1,$2,'items','Expedition preparation refund: 12 Eyes of Ender','{\"id\":\"system.expedition_refund_items\",\"params\":{}}'::jsonb,'escrowed',$3,$4,$5)").bind(asset).bind(actor).bind(&manifest).bind(digest).bind(id).execute(&mut *db).await?;
                    sqlx::query("UPDATE adventures SET material_asset=$2 WHERE id=$1")
                        .bind(adventure)
                        .bind(asset)
                        .execute(&mut *db)
                        .await?;
                    crate::commands::notify(db,actor,"adventure_refund",json!({"adventure_id":adventure,"asset_id":asset,"message":{"id":"text.collect_your_12_eyes_of_ender_from_stored_assets","params":{}}})).await?;
                }
            }
            sqlx::query("UPDATE adventure_participants SET released_at=coalesce(released_at,now()) WHERE adventure_id=$1").bind(adventure).execute(&mut *db).await?;
            if let Some(prepare) = payload
                .get("prepare_job_id")
                .and_then(Value::as_str)
                .and_then(|s| Uuid::parse_str(s).ok())
            {
                sqlx::query("UPDATE jobs SET state='cancelled',updated_at=now() WHERE id=$1 AND state IN ('queued','waiting','leased')").bind(prepare).execute(&mut *db).await?;
            }
        }
        "adventure.close" => {
            receipt(result)?;
            let adventure = uuid(payload, "adventure_id")?;
            if result.get("players_evacuated").and_then(Value::as_bool) != Some(true) {
                return Err(Error::invalid("text.departure_has_not_been_confirmed"));
            }
            sqlx::query("UPDATE worlds SET enabled=false WHERE id=(SELECT world_id FROM adventures WHERE id=$1)").bind(adventure).execute(&mut *db).await?;
            sqlx::query("UPDATE adventures SET state='closed' WHERE id=$1")
                .bind(adventure)
                .execute(&mut *db)
                .await?;
            sqlx::query("UPDATE adventure_participants SET released_at=coalesce(released_at,now()) WHERE adventure_id=$1").bind(adventure).execute(&mut *db).await?;
        }
        "identity.migrate" => {
            receipt(result)?;
            identity::complete(db, actor, payload, result).await?;
        }
        "asset.preview" => {
            if result.get("preview_hash").and_then(Value::as_str).is_none()
                || result.get("clear").and_then(Value::as_bool).is_none()
            {
                return Err(Error::invalid(
                    "text.the_placement_preview_result_is_incomplete",
                ));
            }
        }
        "player.join" => {
            return Err(Error::conflict(
                "text.session_bound_travel_requires_observed_arrival_through_3d6d8c6c52",
            ));
        }
        "adventure.join" | "adventure.return" => {
            receipt(result)?;
            if let Some(session) = payload.get("session") {
                if result.get("session_id") != session.get("session_id") {
                    return Err(Error::invalid(
                        "text.the_expedition_travel_receipt_belongs_to_another_session",
                    ));
                }
            }
        }
        "home.travel" | "player.teleport" | "player.kick" | "server.console" => receipt(result)?,
        "server.logs" => {
            if result.get("lines").and_then(Value::as_array).is_none() {
                return Err(Error::invalid("text.the_log_result_is_missing"));
            }
        }
        _ => {
            return Err(Error::invalid(
                "text.an_unknown_job_cannot_be_accepted_as_successful",
            ));
        }
    }
    Ok(())
}
pub(super) async fn failure(
    db: &mut PgConnection,
    id: Uuid,
    actor: Uuid,
    server: Option<Uuid>,
    kind: &str,
    payload: &Value,
) -> Result<()> {
    match kind {
        "asset.capture" => {
            sqlx::query("UPDATE assets SET state='cancelled',locked_claim_id=NULL WHERE id=$1 AND job_id=$2")
                .bind(uuid(payload, "asset_id")?)
                .bind(id)
                .execute(&mut *db)
                .await?;
        }
        "asset.place" | "asset.receive" => {
            sqlx::query(
                "UPDATE assets SET state='escrowed',locked_claim_id=NULL WHERE id=$1 AND job_id=$2",
            )
            .bind(uuid(payload, "asset_id")?)
            .bind(id)
            .execute(&mut *db)
            .await?;
        }
        "npc.sell" => {
            sqlx::query(
                "UPDATE npc_daily SET coins=coins-$3 WHERE profile_id=$1 AND day=$2::text::date",
            )
            .bind(uuid(payload, "profile_id")?)
            .bind(payload["day"].as_str())
            .bind(
                payload["coins"]
                    .as_i64()
                    .ok_or_else(|| Error::invalid("text.the_amount_is_missing"))?,
            )
            .execute(&mut *db)
            .await?;
        }
        "adventure.prepare" => {
            let changed=sqlx::query("UPDATE adventures SET state='refunded' WHERE id=$1 AND state IN ('preparing','activating')").bind(uuid(payload,"adventure_id")?).execute(&mut *db).await?.rows_affected();
            if changed > 0 {
                sqlx::query("UPDATE wallets SET reserved=reserved-1000 WHERE owner=$1")
                    .bind(actor)
                    .execute(&mut *db)
                    .await?;
                sqlx::query("UPDATE adventure_participants SET released_at=coalesce(released_at,now()) WHERE adventure_id=$1").bind(uuid(payload,"adventure_id")?).execute(&mut *db).await?;
            }
        }
        "claim.sync" => {
            sqlx::query(
                "UPDATE claims SET state='released' WHERE id=$1 AND job_id=$2 AND state='pending'",
            )
            .bind(uuid(payload, "claim_id")?)
            .bind(id)
            .execute(&mut *db)
            .await?;
        }
        "claim.release" => {
            sqlx::query(
                "UPDATE claims SET state='active' WHERE id=$1 AND job_id=$2 AND state='releasing'",
            )
            .bind(uuid(payload, "claim_id")?)
            .bind(id)
            .execute(&mut *db)
            .await?;
        }
        "server.create" | "server.start" | "server.stop" | "server.restore" => {
            // A terminal host failure requires an effect-free or rolled-back receipt.
            // Preserve the last physical observation instead of inventing a VM state.
            sqlx::query("UPDATE servers SET error='{\"id\":\"system.server_operation_failed\",\"params\":{}}' WHERE id=$1").bind(server).execute(&mut *db).await?;
        }
        "server.backup" | "official.backup" => {
            sqlx::query("UPDATE backups SET state='failed',error='{\"id\":\"system.backup_failed\",\"params\":{}}' WHERE id=$1").bind(uuid(payload,"backup_id")?).execute(&mut *db).await?;
        }
        "identity.migrate" => {
            sqlx::query("UPDATE profiles SET status='active' WHERE account_id IN ($1,$2) AND status='moving'").bind(actor).bind(uuid(payload,"other_account")?).execute(&mut *db).await?;
            sqlx::query("UPDATE link_requests SET state='cancelled' WHERE id=$1")
                .bind(uuid(payload, "link_id")?)
                .execute(&mut *db)
                .await?;
        }
        _ => {}
    }
    Ok(())
}
