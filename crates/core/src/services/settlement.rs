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
                return Err(Error::invalid("実サーバーの状態が要求と一致していません。"));
            }
            sqlx::query("UPDATE servers SET observed=$2,last_observed_at=now(),error=NULL,address=coalesce($3,address),capabilities=coalesce($4,capabilities),empty_since=CASE WHEN $2='running' THEN now() ELSE NULL END WHERE id=$1")
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
                return Err(Error::invalid("配置したファイルのハッシュが一致しません。"));
            }
        }
        "server.backup" | "official.backup" => {
            receipt(result)?;
            if result.get("verified").and_then(Value::as_bool) != Some(true) {
                return Err(Error::invalid("バックアップの検証結果がありません。"));
            }
            sqlx::query("UPDATE backups SET state='ready',manifest=$2,error=NULL WHERE id=$1")
                .bind(uuid(payload, "backup_id")?)
                .bind(result)
                .execute(&mut *db)
                .await?;
            if kind == "official.backup" {
                sqlx::query(
                    "UPDATE settings SET value='false' WHERE key='official_mutations_paused'",
                )
                .execute(&mut *db)
                .await?;
            }
        }
        "claim.sync" => {
            receipt(result)?;
            sqlx::query("UPDATE claims SET state='active' WHERE id=$1 AND job_id=$2 AND state IN ('pending','transferring')").bind(uuid(payload,"claim_id")?).bind(id).execute(&mut *db).await?;
            game::reward_event(db, actor, id, "claim.created", 1).await?;
        }
        "claim.release" => {
            receipt(result)?;
            sqlx::query("UPDATE claims SET state='released' WHERE id=$1 AND job_id=$2 AND state='releasing'").bind(uuid(payload,"claim_id")?).bind(id).execute(&mut *db).await?;
        }
        "home.set" => {
            receipt(result)?;
            let location = result
                .get("location")
                .ok_or_else(|| Error::invalid("ホームの位置がありません。"))?;
            if location.get("world_id").and_then(Value::as_str).is_none() {
                return Err(Error::invalid("ホームのワールドがありません。"));
            }
            let profile = uuid(payload, "profile_id")?;
            sqlx::query("SELECT id FROM profiles WHERE id=$1 FOR UPDATE")
                .bind(profile)
                .fetch_one(&mut *db)
                .await?;
            let name = payload["name"]
                .as_str()
                .ok_or_else(|| Error::invalid("ホーム名がありません。"))?;
            let count: i64 =
                sqlx::query_scalar("SELECT count(*) FROM homes WHERE profile_id=$1 AND name<>$2")
                    .bind(profile)
                    .bind(name)
                    .fetch_one(&mut *db)
                    .await?;
            if count >= 3 {
                return Err(Error::conflict("ホームの上限に達しています。"));
            }
            sqlx::query("INSERT INTO homes(id,profile_id,name,location) VALUES($1,$2,$3,$4) ON CONFLICT(profile_id,name) DO UPDATE SET location=$4").bind(Uuid::new_v4()).bind(profile).bind(name).bind(location).execute(&mut *db).await?;
        }
        "asset.capture" => {
            receipt(result)?;
            let asset = uuid(payload, "asset_id")?;
            let manifest = result
                .get("manifest")
                .filter(|m| m.is_object())
                .ok_or_else(|| Error::invalid("資産の保存情報がありません。"))?;
            if uuid(manifest, "asset_id")? != asset {
                return Err(Error::invalid("資産IDが一致しません。"));
            }
            if result.get("original_removed").and_then(Value::as_bool) != Some(true)
                && payload["kind"] != "land"
            {
                return Err(Error::invalid("原本の撤去確認がありません。"));
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
                    .ok_or_else(|| Error::invalid("飼い主IDが不正です。"))?;
                let consent:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM asset_consents WHERE asset_id=$1 AND owner=$2 AND manifest_sha256=$3)").bind(asset).bind(owner).bind(&digest).fetch_one(&mut *db).await?;
                if !consent {
                    return Err(Error::conflict("ペットの飼い主の同意がそろっていません。"));
                }
            }
            sqlx::query("UPDATE assets SET state='escrowed',manifest=$2,manifest_sha256=$3 WHERE id=$1 AND job_id=$4 AND state='capturing'").bind(asset).bind(manifest).bind(digest).bind(id).execute(&mut *db).await?;
        }
        "asset.place" | "asset.receive" => {
            receipt(result)?;
            let asset = uuid(payload, "asset_id")?;
            if uuid(result, "asset_id")? != asset {
                return Err(Error::invalid("受け渡した資産IDが一致しません。"));
            }
            sqlx::query("UPDATE assets SET state=$2 WHERE id=$1 AND job_id=$3 AND state='placing'")
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
                return Err(Error::invalid("取り除いた素材の数が一致しません。"));
            }
            let amount = payload["coins"]
                .as_i64()
                .ok_or_else(|| Error::invalid("買い取り額が不正です。"))?;
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
                    "冒険の準備状態が変わっています。取り消し中なら原状回復を行ってください。",
                ));
            }
            if result.get("world_ready").and_then(Value::as_bool) != Some(true)
                || result.get("eyes_removed").and_then(Value::as_i64) != Some(12)
            {
                return Err(Error::invalid(
                    "ワールド生成と準備アイテムの確保を確認できません。",
                ));
            }
            let world_id = Uuid::new_v4();
            let native = uuid(result, "native_world_id")?;
            let name = result["world_name"]
                .as_str()
                .ok_or_else(|| Error::invalid("ワールド名がありません。"))?;
            sqlx::query("INSERT INTO worlds(id,server_id,name,kind,native_uuid) VALUES($1,$2,$3,'private_end',$4)").bind(world_id).bind(server).bind(name).bind(native).execute(&mut *db).await?;
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
                return Err(Error::invalid("アイテムの返却確認がありません。"));
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
            }
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
                return Err(Error::invalid("退出の確認がありません。"));
            }
            sqlx::query("UPDATE worlds SET enabled=false WHERE id=(SELECT world_id FROM adventures WHERE id=$1)").bind(adventure).execute(&mut *db).await?;
            sqlx::query("UPDATE adventures SET state='closed' WHERE id=$1")
                .bind(adventure)
                .execute(&mut *db)
                .await?;
        }
        "identity.migrate" => {
            receipt(result)?;
            identity::complete(db, actor, payload, result).await?;
        }
        "asset.preview" => {
            if result.get("preview_hash").and_then(Value::as_str).is_none()
                || result.get("clear").and_then(Value::as_bool).is_none()
            {
                return Err(Error::invalid("設置プレビューの結果が不足しています。"));
            }
        }
        "home.travel" | "player.teleport" | "player.join" | "adventure.join" | "player.kick"
        | "server.console" => receipt(result)?,
        "server.logs" => {
            if result.get("lines").and_then(Value::as_array).is_none() {
                return Err(Error::invalid("ログの取得結果がありません。"));
            }
        }
        _ => return Err(Error::invalid("未知のジョブの成功は受理できません。")),
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
            sqlx::query("UPDATE assets SET state='cancelled' WHERE id=$1 AND job_id=$2")
                .bind(uuid(payload, "asset_id")?)
                .bind(id)
                .execute(&mut *db)
                .await?;
        }
        "asset.place" | "asset.receive" => {
            sqlx::query("UPDATE assets SET state='escrowed' WHERE id=$1 AND job_id=$2")
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
                    .ok_or_else(|| Error::invalid("金額がありません。"))?,
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
            sqlx::query("UPDATE servers SET observed='error',error='操作に失敗しました。ジョブの詳細を確認してください。' WHERE id=$1").bind(server).execute(&mut *db).await?;
        }
        "server.backup" | "official.backup" => {
            sqlx::query("UPDATE backups SET state='failed',error='保存に失敗しました。ジョブを確認してください。' WHERE id=$1").bind(uuid(payload,"backup_id")?).execute(&mut *db).await?;
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
