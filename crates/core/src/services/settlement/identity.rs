use super::uuid;
use crate::{
    economy::book,
    error::{Error, Result},
};
use serde_json::{Value, json};
use sqlx::{PgConnection, Row};
use uuid::Uuid;

pub(super) async fn complete(
    db: &mut PgConnection,
    actor: Uuid,
    payload: &Value,
    result: &Value,
) -> Result<()> {
    let link = uuid(payload, "link_id")?;
    let other = uuid(payload, "other_account")?;
    let selected = uuid(payload, "selected_profile")?;
    if result.get("native_data_verified").and_then(Value::as_bool) != Some(true) {
        return Err(Error::invalid("プレイヤーデータの保存検証がありません。"));
    }
    let selected_owner: Uuid = sqlx::query_scalar(
        "SELECT account_id FROM profiles WHERE id=$1 AND status='moving' FOR UPDATE",
    )
    .bind(selected)
    .fetch_one(&mut *db)
    .await?;
    let discarded = if selected_owner == actor {
        other
    } else {
        actor
    };
    let archive = Uuid::new_v4();
    sqlx::query("INSERT INTO principals(id,kind,name) VALUES($1,'system',$2)")
        .bind(archive)
        .bind(format!("連携時アーカイブ {link}"))
        .execute(&mut *db)
        .await?;
    sqlx::query("INSERT INTO wallets(owner) VALUES($1)")
        .bind(archive)
        .execute(&mut *db)
        .await?;
    let balance: i64 = sqlx::query_scalar("SELECT balance FROM wallets WHERE owner=$1 FOR UPDATE")
        .bind(discarded)
        .fetch_one(&mut *db)
        .await?;
    if balance > 0 {
        book(
            db,
            actor,
            &format!("link:{link}:archive"),
            "identity_archive",
            json!({"link":link}),
            &[(discarded, -balance), (archive, balance)],
            false,
        )
        .await?;
    }
    sqlx::query("UPDATE profiles SET status='archived',archive_reason=$3,native_uuid=NULL WHERE account_id IN ($1,$2) AND id<>$4 AND status='moving'").bind(actor).bind(other).bind(format!("link {link}; selected {selected}; archive owner {archive}")).bind(selected).execute(&mut *db).await?;
    sqlx::query("UPDATE assets SET owner=$2 WHERE owner=$1")
        .bind(discarded)
        .bind(archive)
        .execute(&mut *db)
        .await?;
    sqlx::query("UPDATE claims SET owner=$2 WHERE owner=$1")
        .bind(discarded)
        .bind(archive)
        .execute(&mut *db)
        .await?;
    sqlx::query("UPDATE achievement_progress SET owner=$2 WHERE owner=$1")
        .bind(discarded)
        .bind(archive)
        .execute(&mut *db)
        .await?;
    if selected_owner != actor {
        let balance: i64 =
            sqlx::query_scalar("SELECT balance FROM wallets WHERE owner=$1 FOR UPDATE")
                .bind(other)
                .fetch_one(&mut *db)
                .await?;
        if balance > 0 {
            book(
                db,
                actor,
                &format!("link:{link}:selected"),
                "identity_selected",
                json!({"link":link}),
                &[(other, -balance), (actor, balance)],
                false,
            )
            .await?;
        }
        sqlx::query("UPDATE assets SET owner=$2 WHERE owner=$1")
            .bind(other)
            .bind(actor)
            .execute(&mut *db)
            .await?;
        sqlx::query("UPDATE claims SET owner=$2 WHERE owner=$1")
            .bind(other)
            .bind(actor)
            .execute(&mut *db)
            .await?;
        sqlx::query("UPDATE achievement_progress SET owner=$2 WHERE owner=$1")
            .bind(other)
            .bind(actor)
            .execute(&mut *db)
            .await?;
        sqlx::query("UPDATE land_allowances a SET chunks=b.chunks FROM land_allowances b WHERE a.owner=$1 AND b.owner=$2").bind(actor).bind(other).execute(&mut *db).await?;
    }
    sqlx::query("UPDATE profiles SET account_id=$2,status='active',native_uuid=$3 WHERE id=$1")
        .bind(selected)
        .bind(actor)
        .bind(uuid(result, "native_uuid")?)
        .execute(&mut *db)
        .await?;
    sqlx::query("UPDATE identities SET account_id=$2 WHERE account_id=$1")
        .bind(other)
        .bind(actor)
        .execute(&mut *db)
        .await?;
    // Retain one team/party membership. Keep all chat history, including the other membership's room.
    sqlx::query("DELETE FROM team_members WHERE account_id=$1 AND EXISTS(SELECT 1 FROM team_members WHERE account_id=$2)").bind(other).bind(actor).execute(&mut *db).await?;
    sqlx::query("UPDATE team_members SET account_id=$2 WHERE account_id=$1")
        .bind(other)
        .bind(actor)
        .execute(&mut *db)
        .await?;
    sqlx::query("DELETE FROM party_members WHERE account_id=$1 AND EXISTS(SELECT 1 FROM party_members WHERE account_id=$2)").bind(other).bind(actor).execute(&mut *db).await?;
    sqlx::query("UPDATE party_members SET account_id=$2 WHERE account_id=$1")
        .bind(other)
        .bind(actor)
        .execute(&mut *db)
        .await?;
    sqlx::query("UPDATE parties SET leader=$2 WHERE leader=$1")
        .bind(other)
        .bind(actor)
        .execute(&mut *db)
        .await?;
    let dms = sqlx::query("SELECT * FROM direct_rooms WHERE $1 IN (first_id,second_id)")
        .bind(other)
        .fetch_all(&mut *db)
        .await?;
    for dm in dms {
        let a: Uuid = dm.get("first_id");
        let b: Uuid = dm.get("second_id");
        let target = if a == other { b } else { a };
        let room: Uuid = dm.get("room_id");
        sqlx::query("DELETE FROM direct_rooms WHERE room_id=$1")
            .bind(room)
            .execute(&mut *db)
            .await?;
        if target == actor {
            continue;
        }
        let existing:Option<Uuid>=sqlx::query_scalar("SELECT room_id FROM direct_rooms WHERE first_id=least($1,$2) AND second_id=greatest($1,$2)").bind(actor).bind(target).fetch_optional(&mut *db).await?;
        if let Some(existing) = existing {
            sqlx::query("UPDATE messages SET room_id=$2 WHERE room_id=$1")
                .bind(room)
                .bind(existing)
                .execute(&mut *db)
                .await?;
            sqlx::query("UPDATE rooms SET archived_at=now() WHERE id=$1")
                .bind(room)
                .execute(&mut *db)
                .await?;
        } else {
            sqlx::query("INSERT INTO direct_rooms VALUES(least($1,$2),greatest($1,$2),$3)")
                .bind(actor)
                .bind(target)
                .bind(room)
                .execute(&mut *db)
                .await?;
        }
    }
    sqlx::query("INSERT INTO room_members(room_id,account_id,role,joined_at,read_at) SELECT room_id,$2,role,joined_at,read_at FROM room_members WHERE account_id=$1 ON CONFLICT DO NOTHING").bind(other).bind(actor).execute(&mut *db).await?;
    sqlx::query("DELETE FROM room_members WHERE account_id=$1")
        .bind(other)
        .execute(&mut *db)
        .await?;
    sqlx::query("UPDATE rooms SET owner=$2 WHERE owner=$1")
        .bind(other)
        .bind(actor)
        .execute(&mut *db)
        .await?;
    sqlx::query("INSERT INTO friendships(first_id,second_id,requester,state,created_at) SELECT least($2,CASE WHEN first_id=$1 THEN second_id ELSE first_id END),greatest($2,CASE WHEN first_id=$1 THEN second_id ELSE first_id END),CASE WHEN requester=$1 THEN $2 ELSE requester END,state,created_at FROM friendships WHERE $1 IN (first_id,second_id) AND $2 NOT IN (first_id,second_id) ON CONFLICT DO NOTHING").bind(other).bind(actor).execute(&mut *db).await?;
    sqlx::query("DELETE FROM friendships WHERE $1 IN (first_id,second_id)")
        .bind(other)
        .execute(&mut *db)
        .await?;
    sqlx::query("INSERT INTO blocks(actor,target) SELECT CASE WHEN actor=$1 THEN $2 ELSE actor END,CASE WHEN target=$1 THEN $2 ELSE target END FROM blocks WHERE (actor=$1 OR target=$1) AND actor<>$2 AND target<>$2 ON CONFLICT DO NOTHING").bind(other).bind(actor).execute(&mut *db).await?;
    sqlx::query("DELETE FROM blocks WHERE actor=$1 OR target=$1")
        .bind(other)
        .execute(&mut *db)
        .await?;
    sqlx::query("INSERT INTO community_members(community_id,account_id,administrator) SELECT community_id,$2,administrator FROM community_members WHERE account_id=$1 ON CONFLICT DO NOTHING").bind(other).bind(actor).execute(&mut *db).await?;
    sqlx::query("DELETE FROM community_members WHERE account_id=$1")
        .bind(other)
        .execute(&mut *db)
        .await?;
    sqlx::query("UPDATE communities SET owner=$2 WHERE owner=$1")
        .bind(other)
        .bind(actor)
        .execute(&mut *db)
        .await?;
    sqlx::query("UPDATE messages SET author=$2 WHERE author=$1")
        .bind(other)
        .bind(actor)
        .execute(&mut *db)
        .await?;
    sqlx::query("UPDATE notifications SET account_id=$2 WHERE account_id=$1")
        .bind(other)
        .bind(actor)
        .execute(&mut *db)
        .await?;
    sqlx::query("UPDATE accounts SET merged_into=$2 WHERE id=$1")
        .bind(other)
        .bind(actor)
        .execute(&mut *db)
        .await?;
    sqlx::query("DELETE FROM sessions WHERE account_id IN ($1,$2)")
        .bind(actor)
        .bind(other)
        .execute(&mut *db)
        .await?;
    sqlx::query("UPDATE link_requests SET state='complete' WHERE id=$1")
        .bind(link)
        .execute(&mut *db)
        .await?;
    crate::auth::audit(
        db,
        actor,
        "identity.link.complete",
        link,
        json!({"other":other,"selected_profile":selected,"archive":archive}),
    )
    .await?;
    Ok(())
}
