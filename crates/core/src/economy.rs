use crate::{
    auth::{Actor, permission},
    commands::{Command, notify},
    error::{Error, Result},
};
use serde_json::{Value, json};
use sqlx::{PgConnection, Row};
use uuid::Uuid;
pub const TREASURY: Uuid = Uuid::from_u128(1);

pub async fn available(db: &mut PgConnection, owner: Uuid) -> Result<i64> {
    sqlx::query_scalar("SELECT balance-reserved FROM wallets WHERE owner=$1 FOR UPDATE")
        .bind(owner)
        .fetch_optional(db)
        .await?
        .ok_or_else(Error::missing)
}
pub async fn unpaused(db: &mut PgConnection) -> Result<()> {
    let paused: bool = sqlx::query_scalar(
        "SELECT value='true'::jsonb FROM settings WHERE key='official_mutations_paused' FOR SHARE",
    )
    .fetch_one(db)
    .await?;
    if paused {
        return Err(Error::unavailable(
            "公式ワールドの整合性を保つため、保存中は取引を一時停止しています。",
        ));
    }
    Ok(())
}

/// All money writes use a single transaction, sorted wallet locks, and a unique reference.
/// Positive net entries are permitted only in the trusted event reward / NPC settlement paths.
pub async fn book(
    db: &mut PgConnection,
    actor: Uuid,
    reference: &str,
    kind: &str,
    detail: Value,
    legs: &[(Uuid, i64)],
    mint: bool,
) -> Result<Uuid> {
    if let Some(id) = sqlx::query_scalar::<_, Uuid>("SELECT id FROM ledger WHERE reference=$1")
        .bind(reference)
        .fetch_optional(&mut *db)
        .await?
    {
        return Ok(id);
    }
    let mut combined = std::collections::BTreeMap::<Uuid, i64>::new();
    for (owner, amount) in legs {
        let entry = combined.entry(*owner).or_default();
        *entry = entry
            .checked_add(*amount)
            .ok_or_else(|| Error::invalid("金額が大きすぎます。"))?;
    }
    let sum: i128 = combined.values().map(|n| *n as i128).sum();
    if (!mint && sum != 0) || (mint && sum <= 0) {
        return Err(Error::internal("unbalanced ledger request"));
    }
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO ledger(id,reference,kind,actor,detail) VALUES($1,$2,$3,$4,$5)")
        .bind(id)
        .bind(reference)
        .bind(kind)
        .bind(actor)
        .bind(detail)
        .execute(&mut *db)
        .await?;
    for (owner, amount) in combined {
        if amount == 0 {
            continue;
        }
        let spendable = available(db, owner).await?;
        if amount < 0 && spendable < -amount {
            return Err(Error::conflict("利用できる残高が不足しています。"));
        }
        let after: i64 = sqlx::query_scalar(
            "UPDATE wallets SET balance=balance+$2 WHERE owner=$1 RETURNING balance",
        )
        .bind(owner)
        .bind(amount)
        .fetch_one(&mut *db)
        .await?;
        sqlx::query("INSERT INTO ledger_entries(transaction_id,owner,amount,balance_after) VALUES($1,$2,$3,$4)").bind(id).bind(owner).bind(amount).bind(after).execute(&mut *db).await?;
    }
    Ok(id)
}

pub async fn command(
    db: &mut PgConnection,
    actor: &Actor,
    command: &Command,
    request: Uuid,
) -> Result<Value> {
    use Command::*;
    let me = actor.id;
    unpaused(db).await?;
    match command {
        WalletTransfer {
            owner,
            target,
            amount,
        } => {
            let owner = owner.unwrap_or(me);
            permission(db, me, owner, "spend").await?;
            if owner == *target || !(1..=1_000_000_000_000).contains(amount) {
                return Err(Error::invalid("送金先または金額が不正です。"));
            }
            let valid:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM principals p WHERE p.id=$1 AND (p.kind='account' AND EXISTS(SELECT 1 FROM accounts a WHERE a.id=p.id AND a.merged_into IS NULL) OR p.kind='team' AND EXISTS(SELECT 1 FROM teams t WHERE t.id=p.id AND t.disbanded_at IS NULL)))").bind(target).fetch_one(&mut *db).await?;
            if !valid {
                return Err(Error::missing());
            }
            let account_target: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM accounts WHERE id=$1)")
                    .bind(target)
                    .fetch_one(&mut *db)
                    .await?;
            if account_target {
                crate::world::profile(db, *target).await?;
            }
            let id = book(
                db,
                me,
                &format!("transfer:{me}:{request}"),
                "transfer",
                json!({"from":owner,"to":target}),
                &[(owner, -amount), (*target, *amount)],
                false,
            )
            .await?;
            if sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM accounts WHERE id=$1)")
                .bind(target)
                .fetch_one(&mut *db)
                .await?
            {
                notify(
                    db,
                    *target,
                    "transfer",
                    json!({"from":owner,"amount":amount}),
                )
                .await?;
            }
            Ok(json!({"transaction_id":id}))
        }
        ListingCreate { asset, price } => {
            if !(1..=1_000_000_000_000).contains(price) {
                return Err(Error::invalid("価格は1〜1兆コインです。"));
            }
            let row = sqlx::query("SELECT owner,state,kind FROM assets WHERE id=$1 FOR UPDATE")
                .bind(asset)
                .fetch_optional(&mut *db)
                .await?
                .ok_or_else(Error::missing)?;
            let owner: Uuid = row.get("owner");
            permission(db, me, owner, "sell").await?;
            if row.get::<String, _>("state") != "escrowed" {
                return Err(Error::conflict(
                    "原本の預託が完了した資産だけ出品できます。",
                ));
            }
            let id = Uuid::new_v4();
            sqlx::query("INSERT INTO listings(id,asset_id,seller,price) VALUES($1,$2,$3,$4)")
                .bind(id)
                .bind(asset)
                .bind(owner)
                .bind(price)
                .execute(&mut *db)
                .await?;
            sqlx::query("UPDATE assets SET state='listed' WHERE id=$1")
                .bind(asset)
                .execute(db)
                .await?;
            Ok(json!({"listing_id":id}))
        }
        ListingCancel { id } => {
            let row = sqlx::query(
                "SELECT seller,asset_id FROM listings WHERE id=$1 AND state='active' FOR UPDATE",
            )
            .bind(id)
            .fetch_optional(&mut *db)
            .await?
            .ok_or_else(Error::missing)?;
            permission(db, me, row.get("seller"), "sell").await?;
            sqlx::query("UPDATE listings SET state='cancelled' WHERE id=$1")
                .bind(id)
                .execute(&mut *db)
                .await?;
            sqlx::query("UPDATE assets SET state='escrowed' WHERE id=$1 AND state='listed'")
                .bind(row.get::<Uuid, _>("asset_id"))
                .execute(db)
                .await?;
            Ok(json!({"cancelled":true,"asset_id":row.get::<Uuid,_>("asset_id")}))
        }
        ListingBuy { id, owner } => {
            let buyer = owner.unwrap_or(me);
            permission(db, me, buyer, "spend").await?;
            let row=sqlx::query("SELECT l.*,a.kind,a.claim_id FROM listings l JOIN assets a ON a.id=l.asset_id WHERE l.id=$1 AND l.state='active' AND a.state='listed' FOR UPDATE OF l,a").bind(id).fetch_optional(&mut *db).await?.ok_or_else(||Error::conflict("この出品はすでに購入または取り下げされています。"))?;
            let seller: Uuid = row.get("seller");
            if buyer == seller {
                return Err(Error::invalid("自分の出品は購入できません。"));
            }
            let price: i64 = row.get("price");
            let fee = price * 500 / 10000;
            let asset: Uuid = row.get("asset_id");
            if row.get::<String, _>("kind") == "land" {
                permission(db, me, buyer, "build").await?;
                let claim: Uuid = row.get("claim_id");
                let chunks:i32=sqlx::query_scalar("SELECT chunks FROM claims WHERE id=$1 AND owner=$2 AND state='active' FOR UPDATE").bind(claim).bind(seller).fetch_optional(&mut *db).await?.ok_or_else(||Error::conflict("土地の状態が変わっています。"))?;
                crate::world::land_capacity(db, buyer, chunks).await?;
                sqlx::query("UPDATE claims SET owner=$2,state='transferring' WHERE id=$1")
                    .bind(claim)
                    .bind(buyer)
                    .execute(&mut *db)
                    .await?;
                let server = crate::world::official_server(db).await?;
                let job = crate::commands::job(
                    db,
                    me,
                    Some(server),
                    "official",
                    "claim.sync",
                    json!({"claim_id":claim,"asset_id":asset,"buyer_account":me}),
                )
                .await?;
                let job_id =
                    Uuid::parse_str(job["job_id"].as_str().unwrap()).map_err(Error::internal)?;
                sqlx::query("UPDATE claims SET job_id=$2 WHERE id=$1")
                    .bind(claim)
                    .bind(job_id)
                    .execute(&mut *db)
                    .await?;
                crate::hosting::wake(db, me, server).await?;
            }
            let ledger = book(
                db,
                me,
                &format!("purchase:{id}"),
                "market",
                json!({"listing":id,"asset":asset,"fee":fee}),
                &[(buyer, -price), (seller, price - fee), (TREASURY, fee)],
                false,
            )
            .await?;
            let trade = Uuid::new_v4();
            sqlx::query("INSERT INTO trades(id,listing_id,buyer,seller,price,fee,ledger_id) VALUES($1,$2,$3,$4,$5,$6,$7)").bind(trade).bind(id).bind(buyer).bind(seller).bind(price).bind(fee).bind(ledger).execute(&mut *db).await?;
            sqlx::query("UPDATE listings SET state='sold',buyer=$2,sold_at=now() WHERE id=$1")
                .bind(id)
                .bind(buyer)
                .execute(&mut *db)
                .await?;
            sqlx::query("UPDATE assets SET owner=$2,state=CASE WHEN kind='land' THEN 'placed' ELSE 'escrowed' END WHERE id=$1").bind(asset).bind(buyer).execute(&mut *db).await?;
            sqlx::query("INSERT INTO notifications(account_id,kind,body) SELECT id,'market_sale',jsonb_build_object('listing',$2::uuid,'amount',$3::bigint) FROM accounts WHERE id=$1 UNION ALL SELECT account_id,'market_sale',jsonb_build_object('listing',$2::uuid,'amount',$3::bigint) FROM team_members WHERE team_id=$1 AND can_sell").bind(seller).bind(id).bind(price-fee).execute(db).await?;
            Ok(json!({"trade_id":trade,"asset_id":asset,"fee":fee}))
        }
        _ => Err(Error::invalid("この操作は取引機能ではありません。")),
    }
}
