//! Temporary End expeditions. Physical job and receipt identities remain stable.
use crate::{
    auth::Actor,
    commands::Command,
    error::{Error, Result},
    world::{online_official, world_job},
};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::{PgConnection, Row};
use uuid::Uuid;

pub async fn command(db: &mut PgConnection, actor: &Actor, command: &Command) -> Result<Value> {
    use Command::*;
    let me = actor.id;
    match command {
        ExpeditionPrepare => {
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
                // Ready changes lock a member row. Invitations/leaves lock the
                // party, so this ordered roster lock makes consent one snapshot.
                sqlx::query("SELECT account_id FROM party_members WHERE party_id=$1 ORDER BY account_id FOR UPDATE")
                    .bind(party).fetch_all(&mut *db).await?;
                let ready:bool=sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM party_members m WHERE m.party_id=$1 AND (NOT m.ready OR NOT EXISTS(SELECT 1 FROM game_sessions g JOIN servers s ON s.id=g.server_id WHERE g.account_id=m.account_id AND g.lease_until>now() AND s.kind='official' AND (g.combat_until IS NULL OR g.combat_until<=now()))))").bind(party).fetch_one(&mut *db).await?;
                if !ready {
                    return Err(Error::conflict(
                        "text.everyone_must_be_in_the_official_smp_and_marked_ready",
                    ));
                }
            }
            let occupied: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM adventure_participants ap WHERE ap.released_at IS NULL AND (ap.account_id=$1 OR ap.account_id IN (SELECT account_id FROM party_members WHERE party_id=$2)))")
                .bind(me).bind(party_id).fetch_one(&mut *db).await?;
            if occupied {
                return Err(Error::conflict(
                    "text.a_participant_already_has_an_expedition_in_progress",
                ));
            }
            if crate::economy::available(db, me).await? < 1000 {
                return Err(Error::conflict(
                    "text.you_need_1_000_coins_to_prepare_an_adventure",
                ));
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
            sqlx::query("INSERT INTO adventure_participants(adventure_id,account_id) SELECT $1,$2 UNION SELECT $1,account_id FROM party_members WHERE party_id=$3").bind(id).bind(me).bind(party_id).execute(&mut *db).await?;
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
            Ok(json!({"expedition_id":id,"job_id":result["job_id"]}))
        }
        ExpeditionCancel { id } => {
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
                return Err(Error::conflict(
                    "text.an_adventure_cannot_be_cancelled_after_opening",
                ));
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
        ExpeditionEnter { id } => {
            online_official(db, me).await?;
            let world:Uuid=sqlx::query_scalar("SELECT world_id FROM adventures a JOIN adventure_participants ap ON ap.adventure_id=a.id AND ap.account_id=$2 AND ap.released_at IS NULL WHERE a.id=$1 AND a.state='active' AND a.expires_at>now()").bind(id).bind(me).fetch_optional(&mut *db).await?.ok_or_else(Error::forbidden)?;
            let session = travel_session(db, me).await?;
            world_job(
                db,
                me,
                "adventure.join",
                json!({"adventure_id":id,"world_id":world,"session":session}),
            )
            .await
        }
        ExpeditionReturn { id } => {
            online_official(db, me).await?;
            let member: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM adventures a JOIN adventure_participants ap ON ap.adventure_id=a.id WHERE a.id=$1 AND ap.account_id=$2 AND ap.released_at IS NULL AND a.state IN ('active','closing'))")
                .bind(id).bind(me).fetch_one(&mut *db).await?;
            if !member {
                return Err(Error::forbidden());
            }
            let session = travel_session(db, me).await?;
            world_job(
                db,
                me,
                "adventure.return",
                json!({"adventure_id":id,"session":session}),
            )
            .await
        }
        _ => Err(Error::invalid("text.unsupported_expedition_action")),
    }
}

async fn travel_session(db: &mut PgConnection, account: Uuid) -> Result<Value> {
    sqlx::query_scalar("SELECT jsonb_build_object('session_id',g.session_id,'native_uuid',g.native_uuid,'profile_id',g.profile_id) FROM game_sessions g JOIN servers s ON s.id=g.server_id WHERE g.account_id=$1 AND g.lease_until>now() AND s.kind='official'")
        .bind(account).fetch_optional(db).await?.ok_or_else(|| Error::conflict("text.connect_to_the_official_smp_first"))
}

/// The same participant-scoped journal feeds menus, paginated history and detail.
pub async fn rows(
    db: &mut PgConnection,
    account: Uuid,
    limit: i64,
    before: Option<(DateTime<Utc>, Uuid)>,
    only: Option<Uuid>,
) -> Result<Value> {
    let (time, id) = before
        .map(|(time, id)| (Some(time), Some(id)))
        .unwrap_or_default();
    sqlx::query_scalar(
        "SELECT coalesce(jsonb_agg(to_jsonb(a)||jsonb_build_object(
          'destination','end','lifetime','temporary','access','participants',
          'remaining_seconds',greatest(0,extract(epoch FROM (a.expires_at-now()))::bigint),
          'can_cancel',a.owner=$1 AND a.state IN ('preparing','activating'),
          'can_receive',EXISTS(SELECT 1 FROM assets s WHERE s.id=a.material_asset AND s.owner=$1 AND s.state='escrowed'),
          'can_enter',a.state='active' AND a.expires_at>now() AND EXISTS(SELECT 1 FROM adventure_participants ap WHERE ap.adventure_id=a.id AND ap.account_id=$1 AND ap.released_at IS NULL),
          'can_return',a.state IN ('active','closing') AND EXISTS(SELECT 1 FROM adventure_participants ap WHERE ap.adventure_id=a.id AND ap.account_id=$1 AND ap.released_at IS NULL),
          'participants',(SELECT coalesce(jsonb_agg(jsonb_build_object('account_id',ap.account_id,'name',p.name,'consented_at',ap.consented_at,'committed_at',ap.committed_at,'released_at',ap.released_at) ORDER BY p.name,ap.account_id),'[]') FROM adventure_participants ap JOIN principals p ON p.id=ap.account_id WHERE ap.adventure_id=a.id)
          ) ORDER BY a.created_at DESC,a.id DESC),'[]')
        FROM (SELECT a.* FROM adventures a WHERE
          (a.owner=$1 OR EXISTS(SELECT 1 FROM adventure_participants ap WHERE ap.adventure_id=a.id AND ap.account_id=$1))
          AND ($2::timestamptz IS NULL OR (a.created_at,a.id)<($2,$3::uuid))
          AND ($4::uuid IS NULL OR a.id=$4)
          ORDER BY a.created_at DESC,a.id DESC LIMIT $5) a",
    )
    .bind(account).bind(time).bind(id).bind(only).bind(limit.clamp(1, 101))
    .fetch_one(db).await.map_err(Into::into)
}

pub async fn view(db: &mut PgConnection, account: Uuid) -> Result<Value> {
    let journal = rows(db, account, 100, None, None).await?;
    let preparation: Value = sqlx::query_scalar(
        "WITH party AS (SELECT p.id,p.leader FROM parties p JOIN party_members m ON m.party_id=p.id WHERE m.account_id=$1 AND p.closed_at IS NULL),
        roster AS (SELECT $1::uuid AS account_id,coalesce((SELECT ready FROM party_members WHERE account_id=$1),true) AS ready UNION SELECT m.account_id,m.ready FROM party_members m JOIN party p ON p.id=m.party_id),
        participants AS (SELECT r.account_id,p.name,r.ready,
          EXISTS(SELECT 1 FROM game_sessions g JOIN servers s ON s.id=g.server_id WHERE g.account_id=r.account_id AND g.lease_until>now() AND s.kind='official') AS online,
          EXISTS(SELECT 1 FROM accounts a WHERE a.id=r.account_id AND a.combat_until>now()) OR EXISTS(SELECT 1 FROM game_sessions g WHERE g.account_id=r.account_id AND g.combat_until>now()) AS in_combat,
          EXISTS(SELECT 1 FROM adventure_participants ap WHERE ap.account_id=r.account_id AND ap.released_at IS NULL) AS occupied
          FROM roster r JOIN principals p ON p.id=r.account_id)
        SELECT jsonb_build_object('party_id',(SELECT id FROM party),'is_leader',NOT EXISTS(SELECT 1 FROM party WHERE leader<>$1),
          'available_coins',coalesce((SELECT balance-reserved FROM wallets WHERE owner=$1),0),
          'can_prepare',NOT EXISTS(SELECT 1 FROM party WHERE leader<>$1) AND NOT EXISTS(SELECT 1 FROM participants WHERE NOT ready OR NOT online OR in_combat OR occupied) AND coalesce((SELECT balance-reserved FROM wallets WHERE owner=$1),0)>=1000 AND NOT coalesce((SELECT value='true'::jsonb FROM settings WHERE key='official_mutations_paused'),false),
          'participants',(SELECT coalesce(jsonb_agg(to_jsonb(p) ORDER BY p.name,p.account_id),'[]') FROM participants p))",
    ).bind(account).fetch_one(db).await?;
    Ok(
        json!({"expeditions":journal,"preparation":preparation,"cost":{"coins":1000,"ender_eyes":12},"duration_seconds":10800,"destination":"end","lifetime":"temporary","access":"participants"}),
    )
}
