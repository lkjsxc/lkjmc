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
        .ok_or_else(|| Error::unavailable("公式サーバーの登録を待っています。"))
}
pub async fn profile(db: &mut PgConnection, account: Uuid) -> Result<Uuid> {
    sqlx::query_scalar("SELECT id FROM profiles WHERE account_id=$1 AND status='active'")
        .bind(account)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| Error::conflict("プレイデータの移行中です。完了するまでお待ちください。"))
}
pub async fn not_in_combat(db: &mut PgConnection, account: Uuid) -> Result<()> {
    let combat: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM game_sessions WHERE account_id=$1 AND combat_until>now())",
    )
    .bind(account)
    .fetch_one(db)
    .await?;
    if combat {
        return Err(Error::conflict(
            "PvP直後は30秒間、テレポートやサーバー移動ができません。",
        ));
    }
    Ok(())
}
pub async fn online_official(db: &mut PgConnection, account: Uuid) -> Result<Uuid> {
    not_in_combat(db, account).await?;
    sqlx::query_scalar("SELECT s.id FROM game_sessions g JOIN servers s ON s.id=g.server_id WHERE g.account_id=$1 AND g.lease_until>now() AND s.kind='official'").bind(account).fetch_optional(db).await?.ok_or_else(||Error::conflict("公式SMPに接続してから操作してください。"))
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
        return Err(Error::conflict(format!(
            "保護枠が不足しています（使用中 {used} / 上限 {limit} チャンク）。実績で拡張できます。"
        )));
    }
    Ok(())
}
async fn world_job(
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
                return Err(Error::invalid("土地の範囲が不正です。"));
            }
            let area = (*max_x as i64 - *min_x as i64 + 1) * (*max_z as i64 - *min_z as i64 + 1);
            if area > 1_048_576 {
                return Err(Error::invalid("一度に保護する範囲が大きすぎます。"));
            }
            land_capacity(db, owner, area as i32).await?;
            let world: Uuid =
                sqlx::query_scalar("SELECT id FROM worlds WHERE kind='living' AND enabled")
                    .fetch_optional(&mut *db)
                    .await?
                    .ok_or_else(|| Error::unavailable("生活ワールドの準備中です。"))?;
            sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
                .bind(format!("world:{world}"))
                .execute(&mut *db)
                .await?;
            let conflict:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spawn_points p JOIN profiles f ON f.id=p.profile_id WHERE p.world_id=$1 AND p.state IN ('reserved','ready') AND f.account_id<>$2 AND p.x BETWEEN $3::bigint*16-10000 AND ($5::bigint+1)*16+10000 AND p.z BETWEEN $4::bigint*16-10000 AND ($6::bigint+1)*16+10000)")
                .bind(world).bind(me).bind(min_x).bind(min_z).bind(max_x).bind(max_z).fetch_one(&mut *db).await?;
            if conflict {
                return Err(Error::conflict(
                    "近くでプレイヤーの開始地点を準備しています。別の範囲を選んでください。",
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
                return Err(Error::conflict("処理中の土地は解除できません。"));
            }
            let busy:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM assets WHERE claim_id=$1 AND state IN ('capturing','listed','placing','quarantined'))").bind(id).fetch_one(&mut *db).await?;
            if busy {
                return Err(Error::conflict(
                    "建物の処理や出品を完了してから解除してください。",
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
            let count: i64 =
                sqlx::query_scalar("SELECT count(*) FROM homes WHERE profile_id=$1 AND name<>$2")
                    .bind(profile)
                    .bind(name.trim())
                    .fetch_one(&mut *db)
                    .await?;
            if count >= 3 {
                return Err(Error::conflict(
                    "ホームは3つまでです。不要なホームを削除してください。",
                ));
            }
            world_job(
                db,
                me,
                "home.set",
                json!({"name":label(name,32)?,"profile_id":profile}),
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
        TeleportRequest { target } => crate::social::invite(db, me, "teleport", me, *target).await,
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
                return Err(Error::invalid("資産の種類が不正です。"));
            }
            let claim = selection
                .get("claim_id")
                .and_then(Value::as_str)
                .and_then(|s| Uuid::parse_str(s).ok());
            if kind != "items" {
                let claim =
                    claim.ok_or_else(|| Error::invalid("保護した土地を選んでください。"))?;
                let owned:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM claims WHERE id=$1 AND owner=$2 AND state='active')").bind(claim).bind(owner).fetch_one(&mut *db).await?;
                if !owned {
                    return Err(Error::forbidden());
                }
                let busy:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM assets WHERE claim_id=$1 AND state IN ('capturing','placing','quarantined'))").bind(claim).fetch_one(&mut *db).await?;
                if busy {
                    return Err(Error::conflict(
                        "この土地では別の建物処理が進行しています。",
                    ));
                }
            }
            let id = Uuid::new_v4();
            sqlx::query("INSERT INTO assets(id,owner,kind,title,state,claim_id) VALUES($1,$2,$3,$4,'capturing',$5)").bind(id).bind(owner).bind(kind).bind(label(title,100)?).bind(claim).execute(&mut *db).await?;
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
                    "同意対象または建物の内容が変わっています。内容を確認してください。",
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
                return Err(Error::conflict("保管中の建物を選んでください。"));
            }
            let claim = placement
                .get("claim_id")
                .and_then(Value::as_str)
                .and_then(|s| Uuid::parse_str(s).ok())
                .ok_or_else(|| Error::invalid("設置先の土地を選んでください。"))?;
            let plot_owner: Uuid = sqlx::query_scalar(
                "SELECT owner FROM claims WHERE id=$1 AND state='active' FOR UPDATE",
            )
            .bind(claim)
            .fetch_optional(&mut *db)
            .await?
            .ok_or_else(Error::missing)?;
            permission(db, me, plot_owner, "build").await?;
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
                    "設置プレビューを確認してから確定してください。",
                ));
            }
            let result=world_job(db,me,if preview{"asset.preview"}else{"asset.place"},json!({"asset_id":id,"placement":placement,"manifest":row.get::<Value,_>("manifest"),"manifest_sha256":row.get::<Option<String>,_>("manifest_sha256"),"owner":owner})).await?;
            if !preview {
                sqlx::query("UPDATE assets SET state='placing',job_id=$2 WHERE id=$1")
                    .bind(id)
                    .bind(
                        result["job_id"]
                            .as_str()
                            .and_then(|s| Uuid::parse_str(s).ok()),
                    )
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
                return Err(Error::conflict("受け取り可能なアイテムがありません。"));
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
                return Err(Error::invalid("売却数は1〜2,304個です。"));
            }
            let price: i64 =
                sqlx::query_scalar("SELECT price FROM npc_prices WHERE material=$1 AND enabled")
                    .bind(material)
                    .fetch_optional(&mut *db)
                    .await?
                    .ok_or_else(|| Error::invalid("この素材は買い取り対象ではありません。"))?;
            let profile = profile(db, me).await?;
            sqlx::query("INSERT INTO npc_daily(profile_id,day) VALUES($1,(now() AT TIME ZONE 'UTC')::date) ON CONFLICT DO NOTHING").bind(profile).execute(&mut *db).await?;
            let used:i64=sqlx::query_scalar("SELECT coins FROM npc_daily WHERE profile_id=$1 AND day=(now() AT TIME ZONE 'UTC')::date FOR UPDATE").bind(profile).fetch_one(&mut *db).await?;
            let value = price * (*amount as i64);
            if used + value > 2000 {
                return Err(Error::conflict(format!(
                    "本日の残り買い取り枠は{}コインです。UTC 0時に更新します。",
                    2000 - used
                )));
            }
            // Reserve the cap before inventory work; failure releases this exact reservation.
            sqlx::query("UPDATE npc_daily SET coins=coins+$2 WHERE profile_id=$1 AND day=(now() AT TIME ZONE 'UTC')::date").bind(profile).bind(value).execute(&mut *db).await?;
            let day: chrono::NaiveDate =
                sqlx::query_scalar("SELECT (now() AT TIME ZONE 'UTC')::date")
                    .fetch_one(&mut *db)
                    .await?;
            world_job(db,me,"npc.sell",json!({"material":material,"amount":amount,"coins":value,"profile_id":profile,"day":day})).await
        }
        AdventureCreate => {
            crate::economy::unpaused(db).await?;
            online_official(db, me).await?;
            let party=sqlx::query("SELECT p.id,p.leader FROM parties p JOIN party_members m ON m.party_id=p.id WHERE m.account_id=$1 AND p.closed_at IS NULL FOR UPDATE OF p").bind(me).fetch_optional(&mut *db).await?;
            let party_id = if let Some(row) = party {
                if row.get::<Uuid, _>("leader") != me {
                    return Err(Error::forbidden());
                }
                Some(row.get::<Uuid, _>("id"))
            } else {
                None
            };
            if let Some(party) = party_id {
                let ready:bool=sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM party_members m WHERE m.party_id=$1 AND (NOT m.ready OR NOT EXISTS(SELECT 1 FROM game_sessions g JOIN servers s ON s.id=g.server_id WHERE g.account_id=m.account_id AND g.lease_until>now() AND s.kind='official' AND (g.combat_until IS NULL OR g.combat_until<=now()))))").bind(party).fetch_one(&mut *db).await?;
                if !ready {
                    return Err(Error::conflict(
                        "全員が公式SMPに接続し、準備完了にしてください。",
                    ));
                }
            }
            if crate::economy::available(db, me).await? < 1000 {
                return Err(Error::conflict("冒険の準備に1,000コインが必要です。"));
            }
            sqlx::query("UPDATE wallets SET reserved=reserved+1000 WHERE owner=$1")
                .bind(me)
                .execute(&mut *db)
                .await?;
            let id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO adventures(id,owner,party_id,state) VALUES($1,$2,$3,'preparing')",
            )
            .bind(id)
            .bind(me)
            .bind(party_id)
            .execute(&mut *db)
            .await?;
            let result=world_job(db,me,"adventure.prepare",json!({"adventure_id":id,"material":"ENDER_EYE","amount":12,"duration_seconds":10800,"party_id":party_id})).await?;
            sqlx::query("UPDATE adventures SET job_id=$2 WHERE id=$1")
                .bind(id)
                .bind(
                    result["job_id"]
                        .as_str()
                        .and_then(|s| Uuid::parse_str(s).ok()),
                )
                .execute(db)
                .await?;
            Ok(json!({"adventure_id":id,"job_id":result["job_id"]}))
        }
        AdventureCancel { id } => {
            let row = sqlx::query(
                "SELECT state,job_id FROM adventures WHERE id=$1 AND owner=$2 FOR UPDATE",
            )
            .bind(id)
            .bind(me)
            .fetch_optional(&mut *db)
            .await?
            .ok_or_else(Error::missing)?;
            if !matches!(
                row.get::<String, _>("state").as_str(),
                "preparing" | "activating"
            ) {
                return Err(Error::conflict("開いた冒険は取り消せません。"));
            }
            sqlx::query("UPDATE adventures SET state='refunding' WHERE id=$1")
                .bind(id)
                .execute(&mut *db)
                .await?;
            world_job(
                db,
                me,
                "adventure.cancel",
                json!({"adventure_id":id,"prepare_job_id":row.get::<Option<Uuid>,_>("job_id")}),
            )
            .await
        }
        AdventureJoin { id } => {
            online_official(db, me).await?;
            let world:Uuid=sqlx::query_scalar("SELECT world_id FROM adventures a WHERE a.id=$1 AND a.state='active' AND a.expires_at>now() AND (a.owner=$2 OR EXISTS(SELECT 1 FROM party_members m WHERE m.party_id=a.party_id AND m.account_id=$2))").bind(id).bind(me).fetch_optional(&mut *db).await?.ok_or_else(Error::forbidden)?;
            world_job(
                db,
                me,
                "adventure.join",
                json!({"adventure_id":id,"world_id":world}),
            )
            .await
        }
        LinkBegin => {
            let id = Uuid::new_v4();
            let code = random_token()[..12].to_uppercase();
            sqlx::query("INSERT INTO link_requests(id,code_hash,initiator,expires_at) VALUES($1,$2,$3,now()+interval '10 minutes')").bind(id).bind(hash(&code)).bind(me).execute(db).await?;
            Ok(json!({"id":id,"code":code,"expires_in":600}))
        }
        LinkPresent { code } => {
            let row=sqlx::query("UPDATE link_requests SET candidate=$2 WHERE code_hash=$1 AND initiator<>$2 AND candidate IS NULL AND state='pending' AND expires_at>now() RETURNING id,initiator").bind(hash(&code.trim().to_uppercase())).bind(me).fetch_optional(&mut *db).await?.ok_or_else(||Error::invalid("連携コードが無効または期限切れです。"))?;
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
            let p=sqlx::query("SELECT id,account_id FROM profiles WHERE account_id IN ($1,$2) AND status='active' ORDER BY id FOR UPDATE").bind(me).bind(other).fetch_all(&mut *db).await?;
            if p.len() != 2
                || !p
                    .iter()
                    .any(|r| r.get::<Uuid, _>("id") == *selected_profile)
            {
                return Err(Error::invalid("引き継ぐプレイデータを1つ選んでください。"));
            }
            let busy:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM game_sessions WHERE account_id IN ($1,$2) AND lease_until>now()) OR EXISTS(SELECT 1 FROM jobs WHERE actor IN ($1,$2) AND state IN ('queued','leased','waiting')) OR EXISTS(SELECT 1 FROM teams WHERE leader IN ($1,$2) AND disbanded_at IS NULL) OR EXISTS(SELECT 1 FROM adventures WHERE owner IN ($1,$2) AND state NOT IN ('closed','refunded')) OR EXISTS(SELECT 1 FROM listings WHERE seller IN ($1,$2) AND state='active')").bind(me).bind(other).fetch_one(&mut *db).await?;
            if busy {
                return Err(Error::conflict(
                    "両アカウントをゲームから切断し、進行中の処理・出品・冒険を終了してください。チームリーダーは先に委譲してください。",
                ));
            }
            let servers: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM servers WHERE owner=$1)")
                    .bind(other)
                    .fetch_one(&mut *db)
                    .await?;
            if servers {
                return Err(Error::conflict(
                    "連携先が個人サーバーを所有しています。ホスティング枠の移管を管理者に依頼してください。",
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
            let result=world_job(db,me,"identity.migrate",json!({"link_id":id,"retained_account":me,"other_account":other,"selected_profile":selected_profile})).await?;
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
        _ => Err(Error::invalid("この操作はワールド機能ではありません。")),
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
        "first_join" | "death" | "end_portal" | "recovery"
    ) {
        return Err(Error::invalid("再出現の理由が不正です。"));
    }
    let mut tx = app.db.begin().await?;
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
        Error::unavailable("安全な開始地点を再検索しています。隔離待機エリアでお待ちください。")
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
        return Err(Error::invalid("開始地点の結果が不正です。"));
    }
    let mut tx = app.db.begin().await?;
    let profile = profile(&mut tx, request.account_id).await?;
    let n=sqlx::query("UPDATE spawn_points p SET state=$3,y=coalesce($4,y) FROM worlds w WHERE p.id=$1 AND p.profile_id=$2 AND p.world_id=w.id AND w.server_id=$5 AND (p.state=$3 OR p.state='reserved' AND $3 IN ('ready','rejected') OR p.state='ready' AND $3 IN ('used','rejected'))")
        .bind(request.id).bind(profile).bind(&request.state).bind(request.y).bind(service.server_id).execute(&mut *tx).await?.rows_affected();
    if n == 0 {
        return Err(Error::conflict("開始地点の状態が変わっています。"));
    }
    tx.commit().await?;
    Ok(Json(json!({"id":request.id,"state":request.state})))
}
