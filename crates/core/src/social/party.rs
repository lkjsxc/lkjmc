use crate::error::{Error, Result};
use serde_json::{Value, json};
use sqlx::{PgConnection, Row};
use std::sync::OnceLock;
use uuid::Uuid;

/// The command executor holds the account row lock. Duplicate clicks with
/// distinct request IDs converge on the one existing membership and room.
pub(super) async fn create(db: &mut PgConnection, actor: Uuid, name: &str) -> Result<Value> {
    if let Some(row) = sqlx::query("SELECT p.id,p.room_id FROM parties p JOIN party_members m ON m.party_id=p.id WHERE m.account_id=$1 AND p.closed_at IS NULL")
        .bind(actor).fetch_optional(&mut *db).await? {
        return Ok(json!({"party_id":row.get::<Uuid,_>("id"),"room_id":row.get::<Uuid,_>("room_id"),"coalesced":true}));
    }
    let name = if name.trim().is_empty() {
        default_name(db, actor).await?
    } else {
        name.to_owned()
    };
    let room = super::create_room(db, actor, "party", &name).await?;
    let party = Uuid::new_v4();
    sqlx::query("INSERT INTO parties(id,leader,room_id) VALUES($1,$2,$3)")
        .bind(party)
        .bind(actor)
        .bind(room)
        .execute(&mut *db)
        .await?;
    sqlx::query("INSERT INTO party_members(party_id,account_id) VALUES($1,$2)")
        .bind(party)
        .bind(actor)
        .execute(db)
        .await?;
    Ok(json!({"party_id":party,"room_id":room,"name":name}))
}

/// Rename only the current party, and only as its current leader. A website
/// administrator has no extra authority over an unrelated private party.
/// The expected ID fences stale forms after the actor leaves or changes party.
pub(super) async fn rename(
    db: &mut PgConnection,
    actor: Uuid,
    party: Uuid,
    name: &str,
) -> Result<Value> {
    let name = super::label(name, 80)?;
    let row = sqlx::query("SELECT p.id,p.room_id FROM parties p JOIN party_members m ON m.party_id=p.id WHERE m.account_id=$1 AND p.leader=$1 AND p.id=$2 AND p.closed_at IS NULL FOR UPDATE OF p")
        .bind(actor).bind(party).fetch_optional(&mut *db).await?.ok_or_else(Error::forbidden)?;
    let room: Uuid = row.get("room_id");
    let updated = sqlx::query("UPDATE rooms SET name=$2 WHERE id=$1 AND archived_at IS NULL")
        .bind(room)
        .bind(&name)
        .execute(db)
        .await?
        .rows_affected();
    if updated != 1 {
        return Err(Error::missing());
    }
    Ok(json!({"party_id":row.get::<Uuid,_>("id"),"room_id":room,"name":name}))
}

async fn default_name(db: &mut PgConnection, actor: Uuid) -> Result<String> {
    // An editable initial name is persisted once in its creator's language.
    // Shared catalogs remain the source of the default label.
    static CATALOGS: OnceLock<(Value, Value)> = OnceLock::new();
    let (en, ja) = CATALOGS.get_or_init(|| {
        (
            serde_json::from_str(include_str!("../../../../locales/en.json"))
                .expect("English catalog"),
            serde_json::from_str(include_str!("../../../../locales/ja.json"))
                .expect("Japanese catalog"),
        )
    });
    let language: String = sqlx::query_scalar("SELECT language FROM accounts WHERE id=$1")
        .bind(actor)
        .fetch_one(db)
        .await?;
    let catalog = if language == "ja" { ja } else { en };
    Ok(catalog["text.party"]
        .as_str()
        .expect("party label")
        .to_owned())
}

/// Explicit Web/menu targets cannot silently turn into the actor's new party.
/// Legacy clients can omit the target; authorization still resolves and locks
/// their current active membership in the command transaction.
pub(super) async fn scope(
    db: &mut PgConnection,
    actor: Uuid,
    expected: Option<Uuid>,
) -> Result<Uuid> {
    sqlx::query_scalar("SELECT p.id FROM parties p JOIN party_members m ON m.party_id=p.id WHERE m.account_id=$1 AND p.closed_at IS NULL AND ($2::uuid IS NULL OR p.id=$2) FOR UPDATE OF p")
        .bind(actor).bind(expected).fetch_optional(db).await?.ok_or_else(Error::forbidden)
}
