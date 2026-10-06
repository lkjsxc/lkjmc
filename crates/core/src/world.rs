use crate::{
    App,
    auth::{Actor, audit, hash, permission, random_token},
    commands::{Command, job, label, notify},
    error::{Error, Result},
    services::Service,
};
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{PgConnection, Row};
use uuid::Uuid;

pub async fn official_server(db: &mut PgConnection) -> Result<Uuid> {
    sqlx::query_scalar("SELECT id FROM servers WHERE kind='official'")
        .fetch_optional(db)
        .await?
        .ok_or_else(|| Error::unavailable("text.waiting_for_the_official_server_to_be_registered"))
}
pub async fn profile(db: &mut PgConnection, account: Uuid) -> Result<Uuid> {
    sqlx::query_scalar("SELECT id FROM profiles WHERE account_id=$1 AND status='active'")
        .bind(account)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| Error::conflict("text.your_game_data_is_being_transferred_please_wait"))
}
pub async fn not_in_combat(db: &mut PgConnection, account: Uuid) -> Result<()> {
    let combat: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM accounts WHERE id=$1 AND combat_until>now()) OR EXISTS(SELECT 1 FROM game_sessions WHERE account_id=$1 AND combat_until>now())",
    )
    .bind(account)
    .fetch_one(db)
    .await?;
    if combat {
        return Err(Error::conflict(
            "text.teleports_and_server_transfers_are_unavailable_for_30_s_d446208e0e",
        ));
    }
    Ok(())
}
pub async fn online_official(db: &mut PgConnection, account: Uuid) -> Result<Uuid> {
    not_in_combat(db, account).await?;
    sqlx::query_scalar("SELECT s.id FROM game_sessions g JOIN servers s ON s.id=g.server_id WHERE g.account_id=$1 AND g.lease_until>now() AND s.kind='official'").bind(account).fetch_optional(db).await?.ok_or_else(||Error::conflict("text.connect_to_the_official_smp_first"))
}
pub async fn land_capacity(db: &mut PgConnection, owner: Uuid, additional: i32) -> Result<()> {
    let limit: i32 =
        sqlx::query_scalar("SELECT chunks FROM land_allowances WHERE owner=$1 FOR UPDATE")
            .bind(owner)
            .fetch_optional(&mut *db)
            .await?
            .ok_or_else(Error::missing)?;
    let used: i64 = sqlx::query_scalar(
        "SELECT coalesce(sum(chunks),0)::bigint FROM claims WHERE owner=$1 AND state<>'released'",
    )
    .bind(owner)
    .fetch_one(db)
    .await?;
    if used + additional as i64 > limit as i64 {
        return Err(Error::conflict(
            crate::system_message::SystemMessage::new(
                "text.not_enough_claim_allowance_used_used_limit_chunks_earn_07d2ed414a",
            )
            .with("used", used)
            .with("limit", limit),
        ));
    }
    Ok(())
}
pub(crate) async fn world_job(
    db: &mut PgConnection,
    actor: Uuid,
    kind: &str,
    payload: Value,
) -> Result<Value> {
    crate::economy::unpaused(db).await?;
    let server = official_server(db).await?;
    crate::hosting::wake(db, actor, server).await?;
    job(db, actor, Some(server), "official", kind, payload).await
}

pub async fn command(db: &mut PgConnection, actor: &Actor, command: &Command) -> Result<Value> {
    use Command::*;
    let me = actor.id;
    match command {
        ClaimCreate {
            owner,
            name,
            min_x,
            min_z,
            max_x,
            max_z,
        } => {
            crate::economy::unpaused(db).await?;
            let owner = owner.unwrap_or(me);
            permission(db, me, owner, "build").await?;
            if min_x > max_x
                || min_z > max_z
                || [min_x, min_z, max_x, max_z]
                    .iter()
                    .any(|v| v.abs() > 1_800_000)
            {
                return Err(Error::invalid("text.the_claim_bounds_are_invalid"));
            }
            let area = (*max_x as i64 - *min_x as i64 + 1) * (*max_z as i64 - *min_z as i64 + 1);
            if area > 1_048_576 {
                return Err(Error::invalid("text.the_area_is_too_large_for_one_claim"));
            }
            land_capacity(db, owner, area as i32).await?;
            let world: Uuid =
                sqlx::query_scalar("SELECT id FROM worlds WHERE kind='living' AND enabled")
                    .fetch_optional(&mut *db)
                    .await?
                    .ok_or_else(|| {
                        Error::unavailable("text.the_survival_world_is_being_prepared")
                    })?;
            sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
                .bind(format!("world:{world}"))
                .execute(&mut *db)
                .await?;
            let conflict:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spawn_points p JOIN profiles f ON f.id=p.profile_id WHERE p.world_id=$1 AND p.state IN ('reserved','ready') AND f.account_id<>$2 AND p.x BETWEEN $3::bigint*16-10000 AND ($5::bigint+1)*16+10000 AND p.z BETWEEN $4::bigint*16-10000 AND ($6::bigint+1)*16+10000)")
                .bind(world).bind(me).bind(min_x).bind(min_z).bind(max_x).bind(max_z).fetch_one(&mut *db).await?;
            if conflict {
                return Err(Error::conflict(
                    "text.a_player_spawn_is_being_prepared_nearby_choose_another_area",
                ));
            }
            let id = Uuid::new_v4();
            sqlx::query("INSERT INTO claims(id,owner,world_id,name,min_x,min_z,max_x,max_z) VALUES($1,$2,$3,$4,$5,$6,$7,$8)").bind(id).bind(owner).bind(world).bind(label(name,64)?).bind(min_x).bind(min_z).bind(max_x).bind(max_z).execute(&mut *db).await?;
            let result = world_job(db, me, "claim.sync", json!({"claim_id":id})).await?;
            sqlx::query("UPDATE claims SET job_id=$2 WHERE id=$1")
                .bind(id)
                .bind(
                    result["job_id"]
                        .as_str()
                        .and_then(|s| Uuid::parse_str(s).ok()),
                )
                .execute(db)
                .await?;
            Ok(json!({"claim_id":id,"job_id":result["job_id"],"state":"pending"}))
        }
        ClaimRelease { id } => {
            let row = sqlx::query("SELECT owner,state FROM claims WHERE id=$1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *db)
                .await?
                .ok_or_else(Error::missing)?;
            permission(db, me, row.get("owner"), "sell").await?;
            if row.get::<String, _>("state") != "active" {
                return Err(Error::conflict(
                    "text.a_claim_with_an_operation_in_progress_cannot_be_released",
                ));
            }
            let busy: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM assets WHERE locked_claim_id=$1)")
                    .bind(id)
                    .fetch_one(&mut *db)
                    .await?;
            if busy {
                return Err(Error::conflict(
                    "text.finish_building_operations_and_listings_before_releasin_1f44c4fd11",
                ));
            }
            let result = world_job(db, me, "claim.release", json!({"claim_id":id})).await?;
            sqlx::query("UPDATE claims SET state='releasing',job_id=$2 WHERE id=$1")
                .bind(id)
                .bind(
                    result["job_id"]
                        .as_str()
                        .and_then(|s| Uuid::parse_str(s).ok()),
                )
                .execute(db)
                .await?;
            Ok(result)
        }
        HomeSet { name } => {
            online_official(db, me).await?;
            let profile = profile(db, me).await?;
            let name = label(name, 32)?;
            let pending: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM jobs WHERE actor=$1 AND kind='home.set' AND payload->>'name'=$2 AND state IN ('queued','leased','waiting'))")
                .bind(me).bind(&name).fetch_one(&mut *db).await?;
            if pending {
                return Err(Error::conflict(
                    "text.this_home_is_being_registered_please_wait",
                ));
            }
            let count: i64 =
                sqlx::query_scalar("SELECT count(*) FROM (SELECT name FROM homes WHERE profile_id=$1 UNION SELECT payload->>'name' FROM jobs WHERE actor=$3 AND kind='home.set' AND state IN ('queued','leased','waiting')) reserved WHERE name<>$2")
                    .bind(profile)
                    .bind(&name)
                    .bind(me)
                    .fetch_one(&mut *db)
                    .await?;
            if count >= 3 {
                return Err(Error::conflict(
                    "text.you_can_have_up_to_three_homes_delete_an_unneeded_home_first",
                ));
            }
            world_job(
                db,
                me,
                "home.set",
                json!({"name":name,"profile_id":profile}),
            )
            .await
        }
        HomeTravel { id } => {
            online_official(db, me).await?;
            let profile = profile(db, me).await?;
            let location: Value =
                sqlx::query_scalar("SELECT location FROM homes WHERE id=$1 AND profile_id=$2")
                    .bind(id)
                    .bind(profile)
                    .fetch_optional(&mut *db)
                    .await?
                    .ok_or_else(Error::missing)?;
            world_job(
                db,
                me,
                "home.travel",
                json!({"home_id":id,"location":location}),
            )
            .await
        }
        HomeDelete { id } => {
            let profile = profile(db, me).await?;
            let n = sqlx::query("DELETE FROM homes WHERE id=$1 AND profile_id=$2")
                .bind(id)
                .bind(profile)
                .execute(db)
                .await?
                .rows_affected();
            if n == 0 {
                return Err(Error::missing());
            }
            Ok(json!({"deleted":true}))
        }
        TeleportRequest { target, here } => {
            let result = crate::social::invite(db, me, "teleport", me, *target).await?;
            let id = result["id"]
                .as_str()
                .unwrap()
                .parse::<Uuid>()
                .map_err(Error::internal)?;
            sqlx::query("UPDATE invitations SET teleport_here=$2 WHERE id=$1")
                .bind(id)
                .bind(here)
                .execute(&mut *db)
                .await?;
            sqlx::query("UPDATE notifications SET body=body||jsonb_build_object('here',$3::boolean) WHERE account_id=$1 AND kind='invitation' AND body->>'id'=$2")
                .bind(target).bind(id.to_string()).bind(here).execute(&mut *db).await?;
            let expires: chrono::DateTime<chrono::Utc> =
                sqlx::query_scalar("SELECT expires_at FROM invitations WHERE id=$1")
                    .bind(id)
                    .fetch_one(&mut *db)
                    .await?;
            let target_name: String = sqlx::query_scalar("SELECT name FROM principals WHERE id=$1")
                .bind(target)
                .fetch_one(&mut *db)
                .await?;
            Ok(
                json!({"id":id,"here":here,"target":target,"target_name":target_name,"expires_at":expires}),
            )
        }
        AssetCapture {
            owner,
            kind,
            title,
            selection,
            include_contents,
        } => {
            let owner = owner.unwrap_or(me);
            permission(db, me, owner, "sell").await?;
            online_official(db, me).await?;
            if !matches!(kind.as_str(), "items" | "building" | "land") {
                return Err(Error::invalid("text.the_asset_type_is_invalid"));
            }
            let claim = selection
                .get("claim_id")
                .and_then(Value::as_str)
                .and_then(|s| Uuid::parse_str(s).ok());
            if kind != "items" {
                let claim = claim.ok_or_else(|| Error::invalid("text.choose_a_protected_claim"))?;
                let owned: Option<Uuid> = sqlx::query_scalar(
                    "SELECT owner FROM claims WHERE id=$1 AND state='active' FOR UPDATE",
                )
                .bind(claim)
                .fetch_optional(&mut *db)
                .await?;
                if owned != Some(owner) {
                    return Err(Error::forbidden());
                }
                let busy: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM assets WHERE locked_claim_id=$1)",
                )
                .bind(claim)
                .fetch_one(&mut *db)
                .await?;
                if busy {
                    return Err(Error::conflict(
                        "text.another_building_operation_is_in_progress_on_this_claim",
                    ));
                }
            }
            let id = Uuid::new_v4();
            sqlx::query("INSERT INTO assets(id,owner,kind,title,state,claim_id,locked_claim_id) VALUES($1,$2,$3,$4,'capturing',$5,$5)").bind(id).bind(owner).bind(kind).bind(label(title,100)?).bind(if kind=="items" {None} else {claim}).execute(&mut *db).await?;
            let result=world_job(db,me,"asset.capture",json!({"asset_id":id,"kind":kind,"selection":selection,"include_contents":include_contents,"owner":owner})).await?;
            sqlx::query("UPDATE assets SET job_id=$2 WHERE id=$1")
                .bind(id)
                .bind(
                    result["job_id"]
                        .as_str()
                        .and_then(|s| Uuid::parse_str(s).ok()),
                )
                .execute(db)
                .await?;
            Ok(json!({"asset_id":id,"job_id":result["job_id"]}))
        }
        AssetConsent {
            id,
            manifest_sha256,
        } => {
            let valid:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM assets WHERE id=$1 AND manifest_sha256=$2 AND state='capturing' AND manifest->'required_consents' @> to_jsonb(ARRAY[$3::text]))").bind(id).bind(manifest_sha256).bind(me.to_string()).fetch_one(&mut *db).await?;
            if !valid {
                return Err(Error::conflict(
                    "text.the_consent_request_or_building_contents_have_changed_r_ccacd8034a",
                ));
            }
            sqlx::query("INSERT INTO asset_consents(asset_id,owner,manifest_sha256) VALUES($1,$2,$3) ON CONFLICT(asset_id,owner) DO UPDATE SET manifest_sha256=$3,granted_at=now()").bind(id).bind(me).bind(manifest_sha256).execute(&mut *db).await?;
            audit(
                db,
                me,
                "asset.consent",
                id,
                json!({"manifest_sha256":manifest_sha256}),
            )
            .await?;
            Ok(json!({"consented":true}))
        }
        AssetWithdraw { id } => {
            let row=sqlx::query("SELECT a.owner,a.kind,a.state,j.progress FROM assets a LEFT JOIN jobs j ON j.id=a.job_id WHERE a.id=$1 FOR UPDATE OF a")
                .bind(id).fetch_optional(&mut *db).await?.ok_or_else(Error::missing)?;
            permission(db, me, row.get("owner"), "sell").await?;
            if row.get::<String, _>("state") == "capturing" {
                if row
                    .get::<Option<Value>, _>("progress")
                    .as_ref()
                    .and_then(|p| p["phase"].as_str())
                    != Some("awaiting_consent")
                {
                    return Err(Error::conflict(
                        "text.packing_is_in_progress_wait_for_it_to_finish_before_col_b98cf6b3e6",
                    ));
                }
                sqlx::query("UPDATE assets SET cancel_requested=true WHERE id=$1")
                    .bind(id)
                    .execute(&mut *db)
                    .await?;
                return Ok(json!({"cancellation_requested":true}));
            }
            if row.get::<String, _>("kind") != "land" || row.get::<String, _>("state") != "escrowed"
            {
                return Err(Error::conflict(
                    "text.withdraw_the_land_listing_before_releasing_its_deposit",
                ));
            }
            sqlx::query("UPDATE assets SET state='cancelled',locked_claim_id=NULL WHERE id=$1")
                .bind(id)
                .execute(&mut *db)
                .await?;
            Ok(json!({"withdrawn":true}))
        }
        AssetPlace { id, placement } => {
            let row = sqlx::query("SELECT * FROM assets WHERE id=$1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *db)
                .await?
                .ok_or_else(Error::missing)?;
            let owner: Uuid = row.get("owner");
            permission(db, me, owner, "build").await?;
            if row.get::<String, _>("kind") != "building"
                || row.get::<String, _>("state") != "escrowed"
            {
                return Err(Error::conflict("text.choose_a_stored_building"));
            }
            let claim = placement
                .get("claim_id")
                .and_then(Value::as_str)
                .and_then(|s| Uuid::parse_str(s).ok())
                .ok_or_else(|| Error::invalid("text.choose_a_destination_claim"))?;
            let plot_owner: Uuid = sqlx::query_scalar(
                "SELECT owner FROM claims WHERE id=$1 AND state='active' FOR UPDATE",
            )
            .bind(claim)
            .fetch_optional(&mut *db)
            .await?
            .ok_or_else(Error::missing)?;
            permission(db, me, plot_owner, "build").await?;
            let busy: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM assets WHERE locked_claim_id=$1)")
                    .bind(claim)
                    .fetch_one(&mut *db)
                    .await?;
            if busy {
                return Err(Error::conflict(
                    "text.another_operation_is_in_progress_on_the_destination_claim",
                ));
            }
            let preview = placement
                .get("preview")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            if !preview
                && placement
                    .get("preview_hash")
                    .and_then(Value::as_str)
                    .is_none()
            {
                return Err(Error::invalid(
                    "text.review_the_placement_preview_before_confirming",
                ));
            }
            let result=world_job(db,me,if preview{"asset.preview"}else{"asset.place"},json!({"asset_id":id,"placement":placement,"manifest":row.get::<Value,_>("manifest"),"manifest_sha256":row.get::<Option<String>,_>("manifest_sha256"),"owner":owner})).await?;
            if !preview {
                sqlx::query(
                    "UPDATE assets SET state='placing',job_id=$2,locked_claim_id=$3 WHERE id=$1",
                )
                .bind(id)
                .bind(
                    result["job_id"]
                        .as_str()
                        .and_then(|s| Uuid::parse_str(s).ok()),
                )
                .bind(claim)
                .execute(db)
                .await?;
            }
            Ok(result)
        }
        AssetReceive { id } => {
            online_official(db, me).await?;
            let row =
                sqlx::query("SELECT owner,kind,state,manifest FROM assets WHERE id=$1 FOR UPDATE")
                    .bind(id)
                    .fetch_optional(&mut *db)
                    .await?
                    .ok_or_else(Error::missing)?;
            permission(db, me, row.get("owner"), "spend").await?;
            if row.get::<String, _>("kind") != "items"
                || row.get::<String, _>("state") != "escrowed"
            {
                return Err(Error::conflict(
                    "text.there_are_no_items_available_to_collect",
                ));
            }
            let result = world_job(
                db,
                me,
                "asset.receive",
                json!({"asset_id":id,"manifest":row.get::<Value,_>("manifest")}),
            )
            .await?;
            sqlx::query("UPDATE assets SET state='placing',job_id=$2 WHERE id=$1")
                .bind(id)
                .bind(
                    result["job_id"]
                        .as_str()
                        .and_then(|s| Uuid::parse_str(s).ok()),
                )
                .execute(db)
                .await?;
            Ok(result)
        }
        NpcSell { material, amount } => {
            online_official(db, me).await?;
            if !(1..=2304).contains(amount) {
                return Err(Error::invalid("text.sell_between_1_and_2_304_items"));
            }
            let price: i64 =
                sqlx::query_scalar("SELECT price FROM npc_prices WHERE material=$1 AND enabled")
                    .bind(material)
                    .fetch_optional(&mut *db)
                    .await?
                    .ok_or_else(|| {
                        Error::invalid("text.this_material_is_not_accepted_for_buyback")
                    })?;
            let profile = profile(db, me).await?;
            sqlx::query("INSERT INTO npc_daily(profile_id,day) VALUES($1,(now() AT TIME ZONE 'UTC')::date) ON CONFLICT DO NOTHING").bind(profile).execute(&mut *db).await?;
            let used:i64=sqlx::query_scalar("SELECT coins FROM npc_daily WHERE profile_id=$1 AND day=(now() AT TIME ZONE 'UTC')::date FOR UPDATE").bind(profile).fetch_one(&mut *db).await?;
            let value = price * (*amount as i64);
            if used + value > 2000 {
                return Err(Error::conflict(
                    crate::system_message::SystemMessage::new(
                        "text.today_s_remaining_buyback_allowance_is_coins_it_resets_2be4bcae4e",
                    )
                    .with("0", 2000 - used),
                ));
            }
            // Reserve the cap before inventory work; failure releases this exact reservation.
            sqlx::query("UPDATE npc_daily SET coins=coins+$2 WHERE profile_id=$1 AND day=(now() AT TIME ZONE 'UTC')::date").bind(profile).bind(value).execute(&mut *db).await?;
            let day: chrono::NaiveDate =
                sqlx::query_scalar("SELECT (now() AT TIME ZONE 'UTC')::date")
                    .fetch_one(&mut *db)
                    .await?;
            world_job(db,me,"npc.sell",json!({"material":material,"amount":amount,"coins":value,"profile_id":profile,"day":day})).await
        }
        ExpeditionPrepare
        | ExpeditionCancel { .. }
        | ExpeditionEnter { .. }
        | ExpeditionReturn { .. } => crate::expeditions::command(db, actor, command).await,
        LinkBegin => {
            let id = Uuid::new_v4();
            let code = random_token()[..12].to_uppercase();
            sqlx::query("INSERT INTO link_requests(id,code_hash,initiator,expires_at) VALUES($1,$2,$3,now()+interval '10 minutes')").bind(id).bind(hash(&code)).bind(me).execute(db).await?;
            Ok(json!({"id":id,"code":code,"expires_in":600}))
        }
        LinkPresent { code } => {
            let row=sqlx::query("UPDATE link_requests SET candidate=$2 WHERE code_hash=$1 AND initiator<>$2 AND candidate IS NULL AND state='pending' AND expires_at>now() RETURNING id,initiator").bind(hash(&code.trim().to_uppercase())).bind(me).fetch_optional(&mut *db).await?.ok_or_else(||Error::invalid("text.the_link_code_is_invalid_or_has_expired"))?;
            notify(
                db,
                row.get("initiator"),
                "link_candidate",
                json!({"id":row.get::<Uuid,_>("id"),"candidate":me}),
            )
            .await?;
            Ok(json!({"id":row.get::<Uuid,_>("id"),"state":"pending_confirmation"}))
        }
        LinkConfirm {
            id,
            selected_profile,
        } => {
            let row=sqlx::query("SELECT candidate FROM link_requests WHERE id=$1 AND initiator=$2 AND state='pending' AND expires_at>now() AND candidate IS NOT NULL FOR UPDATE").bind(id).bind(me).fetch_optional(&mut *db).await?.ok_or_else(Error::missing)?;
            let other: Uuid = row.get("candidate");
            let eligible: bool = sqlx::query_scalar("SELECT merged_into IS NULL AND NOT administrator AND (banned_until IS NULL OR banned_until<now()) FROM accounts WHERE id=$1")
                .bind(other).fetch_one(&mut *db).await?;
            if !eligible {
                return Err(Error::conflict(
                    "text.the_other_account_is_restricted_merged_or_an_administra_b8982b7f62",
                ));
            }
            let p=sqlx::query("SELECT id,account_id,native_uuid FROM profiles WHERE account_id IN ($1,$2) AND status='active' ORDER BY id FOR UPDATE").bind(me).bind(other).fetch_all(&mut *db).await?;
            if p.len() != 2
                || !p
                    .iter()
                    .any(|r| r.get::<Uuid, _>("id") == *selected_profile)
            {
                return Err(Error::invalid("text.choose_one_set_of_game_data_to_keep"));
            }
            let identities=sqlx::query("SELECT issuer,subject FROM identities WHERE account_id IN ($1,$2) AND issuer IN ('java','bedrock') ORDER BY issuer,subject").bind(me).bind(other).fetch_all(&mut *db).await?;
            for issuer in ["java", "bedrock"] {
                if identities
                    .iter()
                    .filter(|r| r.get::<String, _>("issuer") == issuer)
                    .count()
                    > 1
                {
                    return Err(Error::conflict(
                        "text.you_can_link_one_java_identity_and_one_bedrock_identity",
                    ));
                }
            }
            let native = identities
                .iter()
                .find(|r| r.get::<String, _>("issuer") == "java")
                .map(|r| Uuid::parse_str(&r.get::<String, _>("subject")).map_err(Error::internal))
                .transpose()?
                .or(identities
                    .iter()
                    .find(|r| r.get::<String, _>("issuer") == "bedrock")
                    .map(|r| {
                        r.get::<String, _>("subject")
                            .parse::<u64>()
                            .map(|v| Uuid::from_u128(v as u128))
                            .map_err(Error::internal)
                    })
                    .transpose()?);
            let selected = p
                .iter()
                .find(|r| r.get::<Uuid, _>("id") == *selected_profile)
                .unwrap();
            let discarded = p
                .iter()
                .find(|r| r.get::<Uuid, _>("id") != *selected_profile)
                .unwrap();
            let native_plan = json!({"canonical":native,"selected":selected.get::<Option<Uuid>,_>("native_uuid"),"discarded":discarded.get::<Option<Uuid>,_>("native_uuid"),"archive_owner":Uuid::new_v4(),"discarded_profile":discarded.get::<Uuid,_>("id")});
            not_in_combat(db, me).await?;
            not_in_combat(db, other).await?;
            let busy:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM jobs WHERE actor IN ($1,$2) AND state IN ('queued','leased','waiting')) OR EXISTS(SELECT 1 FROM teams WHERE leader IN ($1,$2) AND disbanded_at IS NULL) OR EXISTS(SELECT 1 FROM parties WHERE leader IN ($1,$2) AND closed_at IS NULL) OR EXISTS(SELECT 1 FROM adventure_participants WHERE account_id IN ($1,$2) AND released_at IS NULL) OR EXISTS(SELECT 1 FROM listings WHERE seller IN ($1,$2) AND state='active')").bind(me).bind(other).fetch_one(&mut *db).await?;
            if busy {
                return Err(Error::conflict(
                    "text.finish_pending_actions_listings_and_adventures_team_and_1939819ef6",
                ));
            }
            let servers: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM servers WHERE owner=$1)")
                    .bind(other)
                    .fetch_one(&mut *db)
                    .await?;
            if servers {
                return Err(Error::conflict(
                    "text.the_other_account_owns_personal_servers_ask_an_administ_72327c46dc",
                ));
            }
            sqlx::query("UPDATE profiles SET status='moving' WHERE account_id IN ($1,$2) AND status='active'").bind(me).bind(other).execute(&mut *db).await?;
            sqlx::query(
                "UPDATE link_requests SET state='migrating',selected_profile=$2 WHERE id=$1",
            )
            .bind(id)
            .bind(selected_profile)
            .execute(&mut *db)
            .await?;
            let result=world_job(db,me,"identity.migrate",json!({"link_id":id,"retained_account":me,"other_account":other,"selected_profile":selected_profile,"native_plan":native_plan})).await?;
            audit(
                db,
                me,
                "identity.link.begin",
                id,
                json!({"candidate":other,"selected_profile":selected_profile}),
            )
            .await?;
            Ok(result)
        }
        _ => Err(Error::invalid("text.this_is_not_a_world_action")),
    }
}

#[derive(Deserialize, Serialize)]
pub struct SpawnReserve {
    pub account_id: Uuid,
    pub reason: String,
}
pub async fn reserve_spawn(
    State(app): State<App>,
    service: Service,
    Json(request): Json<SpawnReserve>,
) -> Result<Json<Value>> {
    service.require("official")?;
    if !matches!(
        request.reason.as_str(),
        "first_join" | "death" | "end_portal" | "recovery" | "adventure_closed"
    ) {
        return Err(Error::invalid("text.the_respawn_reason_is_invalid"));
    }
    let mut tx = app.db.begin().await?;
    crate::economy::unpaused(&mut tx).await?;
    let profile = profile(&mut tx, request.account_id).await?;
    let world: Uuid = sqlx::query_scalar(
        "SELECT id FROM worlds WHERE kind='living' AND enabled AND server_id=$1",
    )
    .bind(service.server_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(Error::forbidden)?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("world:{world}"))
        .execute(&mut *tx)
        .await?;
    if let Some(value)=sqlx::query_scalar::<_,Value>("SELECT to_jsonb(p) FROM spawn_points p WHERE profile_id=$1 AND state IN ('reserved','ready')").bind(profile).fetch_optional(&mut *tx).await?{return Ok(Json(value));}
    let mut candidate = None;
    // This is a reservation only. The Paper adapter must generate the chunk, check hazards and
    // report a safe Y before the point can be used. No default world spawn fallback exists.
    for _ in 0..128 {
        let x = rand::random_range(-250_000i32..=250_000);
        let z = rand::random_range(-250_000i32..=250_000);
        let collision:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spawn_points WHERE world_id=$1 AND state<>'rejected' AND (x::bigint-$2::bigint)^2+(z::bigint-$3::bigint)^2<100000000) OR EXISTS(SELECT 1 FROM claims WHERE world_id=$1 AND state<>'released' AND greatest(min_x::bigint*16-$2::bigint,0,$2::bigint-(max_x::bigint*16+15))^2+greatest(min_z::bigint*16-$3::bigint,0,$3::bigint-(max_z::bigint*16+15))^2<100000000)")
            .bind(world).bind(x).bind(z).fetch_one(&mut *tx).await?;
        if !collision {
            candidate = Some((x, z));
            break;
        }
    }
    let (x, z) = candidate.ok_or_else(|| {
        Error::unavailable("text.searching_for_another_safe_spawn_please_wait_in_the_holding_area")
    })?;
    let id = Uuid::new_v4();
    let value:Value=sqlx::query_scalar("INSERT INTO spawn_points(id,profile_id,world_id,x,z,state,reason) VALUES($1,$2,$3,$4,$5,'reserved',$6) RETURNING to_jsonb(spawn_points)").bind(id).bind(profile).bind(world).bind(x).bind(z).bind(request.reason).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(value))
}
#[derive(Deserialize)]
pub struct SpawnResolve {
    id: Uuid,
    account_id: Uuid,
    y: Option<i32>,
    state: String,
}
pub async fn resolve_spawn(
    State(app): State<App>,
    service: Service,
    Json(request): Json<SpawnResolve>,
) -> Result<Json<Value>> {
    service.require("official")?;
    if !matches!(request.state.as_str(), "ready" | "used" | "rejected")
        || (request.state == "ready" && !request.y.is_some_and(|y| (-64..=320).contains(&y)))
    {
        return Err(Error::invalid("text.the_spawn_result_is_invalid"));
    }
    let mut tx = app.db.begin().await?;
    crate::economy::unpaused(&mut tx).await?;
    let profile = profile(&mut tx, request.account_id).await?;
    let n=sqlx::query("UPDATE spawn_points p SET state=$3,y=coalesce($4,y) FROM worlds w WHERE p.id=$1 AND p.profile_id=$2 AND p.world_id=w.id AND w.server_id=$5 AND (p.state=$3 OR p.state='reserved' AND $3 IN ('ready','rejected') OR p.state='ready' AND $3 IN ('used','rejected'))")
        .bind(request.id).bind(profile).bind(&request.state).bind(request.y).bind(service.server_id).execute(&mut *tx).await?.rows_affected();
    if n == 0 {
        return Err(Error::conflict("text.the_spawn_state_has_changed"));
    }
    tx.commit().await?;
    Ok(Json(json!({"id":request.id,"state":request.state})))
}
