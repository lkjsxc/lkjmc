use crate::{
    auth::{Actor, audit, permission},
    commands::{Command, label, notify},
    error::{Error, Result},
};
use serde_json::{Value, json};
use sqlx::{PgConnection, Row};
use uuid::Uuid;

pub async fn unblocked(db: &mut PgConnection, a: Uuid, b: Uuid) -> Result<()> {
    let active:bool=sqlx::query_scalar("SELECT count(*)=2 FROM accounts a JOIN profiles p ON p.account_id=a.id AND p.status='active' WHERE a.id IN ($1,$2) AND a.merged_into IS NULL").bind(a).bind(b).fetch_one(&mut *db).await?;
    let blocked:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM blocks WHERE (actor=$1 AND target=$2) OR (actor=$2 AND target=$1))").bind(a).bind(b).fetch_one(db).await?;
    if a == b || blocked || !active {
        return Err(Error::forbidden());
    }
    Ok(())
}
pub async fn friends(db: &mut PgConnection, a: Uuid, b: Uuid) -> Result<bool> {
    Ok(sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM friendships WHERE first_id=least($1,$2) AND second_id=greatest($1,$2) AND state='accepted')").bind(a).bind(b).fetch_one(db).await?)
}
pub async fn room_member(db: &mut PgConnection, room: Uuid, actor: Uuid) -> Result<String> {
    sqlx::query_scalar("SELECT m.role FROM room_members m JOIN rooms r ON r.id=m.room_id WHERE m.room_id=$1 AND m.account_id=$2 AND r.archived_at IS NULL")
        .bind(room).bind(actor).fetch_optional(db).await?.ok_or_else(Error::forbidden)
}
async fn create_room(db: &mut PgConnection, actor: Uuid, kind: &str, name: &str) -> Result<Uuid> {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO rooms(id,kind,name,owner) VALUES($1,$2,$3,$4)")
        .bind(id)
        .bind(kind)
        .bind(label(name, 80)?)
        .bind(actor)
        .execute(&mut *db)
        .await?;
    sqlx::query("INSERT INTO room_members(room_id,account_id,role) VALUES($1,$2,'owner')")
        .bind(id)
        .bind(actor)
        .execute(db)
        .await?;
    Ok(id)
}
pub async fn invite(
    db: &mut PgConnection,
    actor: Uuid,
    kind: &str,
    resource: Uuid,
    target: Uuid,
) -> Result<Value> {
    unblocked(db, actor, target).await?;
    match kind {
        "room" => {
            let role = room_member(db, resource, actor).await?;
            let group: bool = sqlx::query_scalar("SELECT kind='group' FROM rooms WHERE id=$1")
                .bind(resource)
                .fetch_one(&mut *db)
                .await?;
            if !group || !matches!(role.as_str(), "owner" | "moderator") {
                return Err(Error::forbidden());
            }
        }
        "team" => permission(db, actor, resource, "members").await?,
        "party" => {
            let allowed:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM parties WHERE id=$1 AND leader=$2 AND closed_at IS NULL)").bind(resource).bind(actor).fetch_one(&mut *db).await?;
            if !allowed {
                return Err(Error::forbidden());
            }
        }
        "community" => {
            let allowed:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM communities c WHERE id=$1 AND (owner=$2 OR EXISTS(SELECT 1 FROM community_members m WHERE m.community_id=c.id AND m.account_id=$2 AND m.administrator)))").bind(resource).bind(actor).fetch_one(&mut *db).await?;
            if !allowed {
                return Err(Error::forbidden());
            }
        }
        "server" => {
            crate::hosting::server_permission(db, actor, resource, true).await?;
        }
        "teleport" => {
            if resource != actor {
                return Err(Error::forbidden());
            }
            crate::world::online_official(db, actor).await?;
            crate::world::online_official(db, target).await?;
        }
        _ => return Err(Error::invalid("招待の種類が不正です。")),
    }
    let id = Uuid::new_v4();
    sqlx::query("UPDATE invitations SET state='cancelled' WHERE recipient=$1 AND kind=$2 AND resource_id=$3 AND state='pending' AND expires_at<=now()")
        .bind(target).bind(kind).bind(resource).execute(&mut *db).await?;
    sqlx::query("INSERT INTO invitations(id,sender,recipient,kind,resource_id,expires_at) VALUES($1,$2,$3,$4,$5,now()+CASE WHEN $4='teleport' THEN interval '2 minutes' ELSE interval '7 days' END)")
        .bind(id).bind(actor).bind(target).bind(kind).bind(resource).execute(&mut *db).await?;
    notify(
        db,
        target,
        "invitation",
        json!({"id":id,"kind":kind,"sender":actor}),
    )
    .await?;
    Ok(json!({"id":id}))
}

async fn respond(db: &mut PgConnection, actor: Uuid, id: Uuid, accept: bool) -> Result<Value> {
    let row=sqlx::query("SELECT * FROM invitations WHERE id=$1 AND recipient=$2 AND state='pending' AND expires_at>now() FOR UPDATE").bind(id).bind(actor).fetch_optional(&mut *db).await?.ok_or_else(Error::missing)?;
    let sender: Uuid = row.get("sender");
    let resource: Uuid = row.get("resource_id");
    let kind: String = row.get("kind");
    let mut result = json!({"accepted":accept});
    if accept {
        unblocked(db, actor, sender).await?;
        // Recheck the inviter's authority at acceptance; invites cannot outlive their authority.
        let room = match kind.as_str() {
            "room" => {
                let role = room_member(db, resource, sender).await?;
                if !matches!(role.as_str(), "owner" | "moderator") {
                    return Err(Error::forbidden());
                }
                Some(resource)
            }
            "team" => {
                permission(db, sender, resource, "members").await?;
                sqlx::query("SELECT id FROM teams WHERE id=$1 AND disbanded_at IS NULL FOR UPDATE")
                    .bind(resource)
                    .fetch_optional(&mut *db)
                    .await?
                    .ok_or_else(Error::missing)?;
                sqlx::query("INSERT INTO team_members(team_id,account_id) VALUES($1,$2)")
                    .bind(resource)
                    .bind(actor)
                    .execute(&mut *db)
                    .await?;
                Some(
                    sqlx::query_scalar::<_, Uuid>("SELECT room_id FROM teams WHERE id=$1")
                        .bind(resource)
                        .fetch_one(&mut *db)
                        .await?,
                )
            }
            "party" => {
                let room:Uuid=sqlx::query_scalar("SELECT room_id FROM parties WHERE id=$1 AND leader=$2 AND closed_at IS NULL FOR UPDATE").bind(resource).bind(sender).fetch_optional(&mut *db).await?.ok_or_else(Error::forbidden)?;
                sqlx::query("INSERT INTO party_members(party_id,account_id) VALUES($1,$2)")
                    .bind(resource)
                    .bind(actor)
                    .execute(&mut *db)
                    .await?;
                Some(room)
            }
            "community" => {
                let allowed:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM communities c WHERE c.id=$1 AND (owner=$2 OR EXISTS(SELECT 1 FROM community_members m WHERE m.community_id=c.id AND m.account_id=$2 AND m.administrator)))").bind(resource).bind(sender).fetch_one(&mut *db).await?;
                if !allowed {
                    return Err(Error::forbidden());
                }
                sqlx::query("INSERT INTO community_members(community_id,account_id) VALUES($1,$2) ON CONFLICT DO NOTHING").bind(resource).bind(actor).execute(&mut *db).await?;
                None
            }
            "server" => {
                crate::hosting::server_permission(db, sender, resource, true).await?;
                sqlx::query("INSERT INTO server_members(server_id,account_id,role) VALUES($1,$2,'guest') ON CONFLICT DO NOTHING").bind(resource).bind(actor).execute(&mut *db).await?;
                None
            }
            "teleport" => {
                let server = crate::world::online_official(db, sender).await?;
                crate::world::online_official(db, actor).await?;
                result = crate::commands::job(
                    db,
                    sender,
                    Some(server),
                    "official",
                    "player.teleport",
                    json!({"target":actor,"accepted_by":actor,"invitation":id}),
                )
                .await?;
                None
            }
            _ => return Err(Error::invalid("不明な招待です。")),
        };
        if let Some(room) = room {
            sqlx::query(
                "INSERT INTO room_members(room_id,account_id) VALUES($1,$2) ON CONFLICT DO NOTHING",
            )
            .bind(room)
            .bind(actor)
            .execute(&mut *db)
            .await?;
        }
    }
    sqlx::query("UPDATE invitations SET state=$2 WHERE id=$1")
        .bind(id)
        .bind(if accept { "accepted" } else { "declined" })
        .execute(&mut *db)
        .await?;
    notify(
        db,
        sender,
        "invitation_response",
        json!({"id":id,"accepted":accept,"account":actor}),
    )
    .await?;
    Ok(result)
}

pub async fn command(db: &mut PgConnection, actor: &Actor, command: &Command) -> Result<Value> {
    use Command::*;
    let me = actor.id;
    match command {
        FriendRequest { target } => {
            unblocked(db, me, *target).await?;
            sqlx::query("INSERT INTO friendships(first_id,second_id,requester,state) VALUES(least($1,$2),greatest($1,$2),$1,'pending')").bind(me).bind(target).execute(&mut *db).await?;
            notify(db, *target, "friend_request", json!({"account":me})).await?;
            Ok(json!({"requested":true}))
        }
        FriendRespond { target, accept } => {
            unblocked(db, me, *target).await?;
            let n = if *accept {
                sqlx::query("UPDATE friendships SET state='accepted' WHERE first_id=least($1,$2) AND second_id=greatest($1,$2) AND requester=$2 AND state='pending'").bind(me).bind(target).execute(&mut *db).await?.rows_affected()
            } else {
                sqlx::query("DELETE FROM friendships WHERE first_id=least($1,$2) AND second_id=greatest($1,$2) AND requester=$2 AND state='pending'").bind(me).bind(target).execute(&mut *db).await?.rows_affected()
            };
            if n == 0 {
                return Err(Error::missing());
            }
            notify(
                db,
                *target,
                "friend_response",
                json!({"account":me,"accepted":accept}),
            )
            .await?;
            Ok(json!({"accepted":accept}))
        }
        FriendRemove { target } => {
            sqlx::query(
                "DELETE FROM friendships WHERE first_id=least($1,$2) AND second_id=greatest($1,$2)",
            )
            .bind(me)
            .bind(target)
            .execute(db)
            .await?;
            Ok(json!({"removed":true}))
        }
        DirectRoom { target } => {
            unblocked(db, me, *target).await?;
            let policy: String = sqlx::query_scalar(
                "SELECT dm_policy FROM accounts WHERE id=$1 AND merged_into IS NULL",
            )
            .bind(target)
            .fetch_optional(&mut *db)
            .await?
            .ok_or_else(Error::missing)?;
            if policy == "none" || (policy == "friends" && !friends(db, me, *target).await?) {
                return Err(Error::forbidden());
            }
            sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
                .bind(format!("dm:{}:{}", me.min(*target), me.max(*target)))
                .execute(&mut *db)
                .await?;
            if let Some(id)=sqlx::query_scalar::<_,Uuid>("SELECT room_id FROM direct_rooms WHERE first_id=least($1,$2) AND second_id=greatest($1,$2)").bind(me).bind(target).fetch_optional(&mut *db).await? {return Ok(json!({"room_id":id}));}
            let id = create_room(db, me, "dm", "個別チャット").await?;
            sqlx::query("INSERT INTO direct_rooms(first_id,second_id,room_id) VALUES(least($1,$2),greatest($1,$2),$3)").bind(me).bind(target).bind(id).execute(&mut *db).await?;
            sqlx::query("INSERT INTO room_members(room_id,account_id) VALUES($1,$2)")
                .bind(id)
                .bind(target)
                .execute(db)
                .await?;
            Ok(json!({"room_id":id}))
        }
        RoomCreate { name } => Ok(json!({"room_id":create_room(db,me,"group",name).await?})),
        MessageSend { room, body } => {
            room_member(db, *room, me).await?;
            let room_kind: String = sqlx::query_scalar("SELECT kind FROM rooms WHERE id=$1")
                .bind(room)
                .fetch_one(&mut *db)
                .await?;
            if room_kind == "dm" {
                let target: Uuid = sqlx::query_scalar(
                    "SELECT account_id FROM room_members WHERE room_id=$1 AND account_id<>$2",
                )
                .bind(room)
                .bind(me)
                .fetch_one(&mut *db)
                .await?;
                unblocked(db, me, target).await?;
                let policy: String =
                    sqlx::query_scalar("SELECT dm_policy FROM accounts WHERE id=$1")
                        .bind(target)
                        .fetch_one(&mut *db)
                        .await?;
                if policy == "none" || (policy == "friends" && !friends(db, me, target).await?) {
                    return Err(Error::forbidden());
                }
            }
            let count:i64=sqlx::query_scalar("SELECT count(*) FROM messages WHERE author=$1 AND created_at>now()-interval '1 minute'").bind(me).fetch_one(&mut *db).await?;
            if count >= 30 {
                return Err(Error::conflict(
                    "メッセージが多すぎます。少し待ってください。",
                ));
            }
            let id: i64 = sqlx::query_scalar(
                "INSERT INTO messages(room_id,author,body) VALUES($1,$2,$3) RETURNING id",
            )
            .bind(room)
            .bind(me)
            .bind(label(body, 4000)?)
            .fetch_one(&mut *db)
            .await?;
            sqlx::query("INSERT INTO notifications(account_id,kind,body) SELECT m.account_id,'message',jsonb_build_object('room',$1::uuid,'message_id',$2::bigint) FROM room_members m WHERE m.room_id=$1 AND m.account_id<>$3 AND NOT EXISTS(SELECT 1 FROM blocks b WHERE (b.actor=m.account_id AND b.target=$3) OR (b.target=m.account_id AND b.actor=$3))")
                .bind(room).bind(id).bind(me).execute(db).await?;
            Ok(json!({"message_id":id}))
        }
        MessageDelete { id } => {
            let n = sqlx::query(
                "UPDATE messages SET body='',deleted_at=now() WHERE id=$1 AND author=$2",
            )
            .bind(id)
            .bind(me)
            .execute(db)
            .await?
            .rows_affected();
            if n == 0 {
                return Err(Error::missing());
            }
            Ok(json!({"deleted":true}))
        }
        RoomRead { room } => {
            room_member(db, *room, me).await?;
            sqlx::query("UPDATE room_members SET read_at=now() WHERE room_id=$1 AND account_id=$2")
                .bind(room)
                .bind(me)
                .execute(db)
                .await?;
            Ok(json!({"read":true}))
        }
        RoomLeave { room } => {
            room_member(db, *room, me).await?;
            let row = sqlx::query("SELECT kind,owner FROM rooms WHERE id=$1 FOR UPDATE")
                .bind(room)
                .fetch_one(&mut *db)
                .await?;
            if row.get::<String, _>("kind") != "group" {
                return Err(Error::invalid(
                    "チーム・パーティーは所属の画面から退出してください。",
                ));
            }
            if row.get::<Option<Uuid>, _>("owner") == Some(me) {
                let next:Option<Uuid>=sqlx::query_scalar("SELECT account_id FROM room_members WHERE room_id=$1 AND account_id<>$2 ORDER BY joined_at LIMIT 1").bind(room).bind(me).fetch_optional(&mut *db).await?;
                sqlx::query("UPDATE rooms SET owner=$2,archived_at=CASE WHEN $2::uuid IS NULL THEN now() ELSE NULL END WHERE id=$1").bind(room).bind(next).execute(&mut *db).await?;
                if let Some(next) = next {
                    sqlx::query(
                        "UPDATE room_members SET role='owner' WHERE room_id=$1 AND account_id=$2",
                    )
                    .bind(room)
                    .bind(next)
                    .execute(&mut *db)
                    .await?;
                }
            }
            sqlx::query("DELETE FROM room_members WHERE room_id=$1 AND account_id=$2")
                .bind(room)
                .bind(me)
                .execute(db)
                .await?;
            Ok(json!({"left":true}))
        }
        TeamCreate { name } => {
            let name = label(name, 64)?;
            let team = Uuid::new_v4();
            let room = create_room(db, me, "team", &name).await?;
            sqlx::query("INSERT INTO principals(id,kind,name) VALUES($1,'team',$2)")
                .bind(team)
                .bind(name)
                .execute(&mut *db)
                .await?;
            sqlx::query("INSERT INTO teams(id,leader,room_id) VALUES($1,$2,$3)")
                .bind(team)
                .bind(me)
                .bind(room)
                .execute(&mut *db)
                .await?;
            sqlx::query("INSERT INTO team_members(team_id,account_id,can_build,can_sell,can_spend,can_manage_members,can_administer) VALUES($1,$2,true,true,true,true,true)").bind(team).bind(me).execute(&mut *db).await?;
            sqlx::query("INSERT INTO land_allowances(owner,chunks) VALUES($1,16)")
                .bind(team)
                .execute(&mut *db)
                .await?;
            sqlx::query("INSERT INTO wallets(owner) VALUES($1)")
                .bind(team)
                .execute(db)
                .await?;
            Ok(json!({"team_id":team,"room_id":room}))
        }
        TeamPermissions {
            team,
            member,
            build,
            sell,
            spend,
            members,
            administer,
        } => {
            permission(db, me, *team, "admin").await?;
            let leader: Uuid =
                sqlx::query_scalar("SELECT leader FROM teams WHERE id=$1 FOR UPDATE")
                    .bind(team)
                    .fetch_one(&mut *db)
                    .await?;
            if *member == leader {
                return Err(Error::invalid("リーダーの権限は委譲で変更します。"));
            }
            if *administer && leader != me {
                return Err(Error::forbidden());
            }
            let n=sqlx::query("UPDATE team_members SET can_build=$3,can_sell=$4,can_spend=$5,can_manage_members=$6,can_administer=$7 WHERE team_id=$1 AND account_id=$2").bind(team).bind(member).bind(build).bind(sell).bind(spend).bind(members).bind(administer).execute(&mut *db).await?.rows_affected();
            if n == 0 {
                return Err(Error::missing());
            }
            audit(db, me, "team.permissions", team, json!({"member":member})).await?;
            Ok(json!({"updated":true}))
        }
        TeamTransfer { team, target } => {
            let n=sqlx::query("UPDATE teams SET leader=$3 WHERE id=$1 AND leader=$2 AND EXISTS(SELECT 1 FROM team_members WHERE team_id=$1 AND account_id=$3)").bind(team).bind(me).bind(target).execute(&mut *db).await?.rows_affected();
            if n == 0 {
                return Err(Error::forbidden());
            }
            sqlx::query(
                "UPDATE team_members SET can_administer=false WHERE team_id=$1 AND account_id=$2",
            )
            .bind(team)
            .bind(me)
            .execute(&mut *db)
            .await?;
            sqlx::query(
                "UPDATE rooms SET owner=$2 WHERE id=(SELECT room_id FROM teams WHERE id=$1)",
            )
            .bind(team)
            .bind(target)
            .execute(&mut *db)
            .await?;
            audit(db, me, "team.transfer", team, json!({"leader":target})).await?;
            Ok(json!({"transferred":true}))
        }
        TeamLeave => {
            let row=sqlx::query("SELECT t.id,t.leader,t.room_id FROM teams t JOIN team_members m ON m.team_id=t.id WHERE m.account_id=$1 FOR UPDATE OF t").bind(me).fetch_optional(&mut *db).await?.ok_or_else(Error::missing)?;
            if row.get::<Uuid, _>("leader") == me {
                return Err(Error::conflict(
                    "先にリーダーを委譲するか、資産を処分してチームを解散してください。",
                ));
            }
            sqlx::query("DELETE FROM team_members WHERE account_id=$1")
                .bind(me)
                .execute(&mut *db)
                .await?;
            sqlx::query("DELETE FROM room_members WHERE room_id=$1 AND account_id=$2")
                .bind(row.get::<Uuid, _>("room_id"))
                .bind(me)
                .execute(db)
                .await?;
            Ok(json!({"left":true}))
        }
        TeamDisband { team } => {
            let row=sqlx::query("SELECT room_id FROM teams WHERE id=$1 AND leader=$2 AND disbanded_at IS NULL FOR UPDATE").bind(team).bind(me).fetch_optional(&mut *db).await?.ok_or_else(Error::forbidden)?;
            let has_assets:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM claims WHERE owner=$1 AND state<>'released') OR EXISTS(SELECT 1 FROM assets WHERE owner=$1 AND state NOT IN ('delivered','placed')) OR EXISTS(SELECT 1 FROM wallets WHERE owner=$1 AND balance>0)").bind(team).fetch_one(&mut *db).await?;
            if has_assets {
                return Err(Error::conflict(
                    "土地・保管資産・共有残高を先に処分してください。",
                ));
            }
            sqlx::query("UPDATE teams SET disbanded_at=now() WHERE id=$1")
                .bind(team)
                .execute(&mut *db)
                .await?;
            sqlx::query("DELETE FROM team_members WHERE team_id=$1")
                .bind(team)
                .execute(&mut *db)
                .await?;
            sqlx::query("UPDATE rooms SET archived_at=now() WHERE id=$1")
                .bind(row.get::<Uuid, _>("room_id"))
                .execute(db)
                .await?;
            Ok(json!({"disbanded":true}))
        }
        PartyCreate { name } => {
            let room = create_room(db, me, "party", name).await?;
            let party = Uuid::new_v4();
            sqlx::query("INSERT INTO parties(id,leader,room_id) VALUES($1,$2,$3)")
                .bind(party)
                .bind(me)
                .bind(room)
                .execute(&mut *db)
                .await?;
            sqlx::query("INSERT INTO party_members(party_id,account_id) VALUES($1,$2)")
                .bind(party)
                .bind(me)
                .execute(db)
                .await?;
            Ok(json!({"party_id":party,"room_id":room}))
        }
        PartyReady { ready } => {
            let n = sqlx::query("UPDATE party_members SET ready=$2 WHERE account_id=$1")
                .bind(me)
                .bind(ready)
                .execute(db)
                .await?
                .rows_affected();
            if n == 0 {
                return Err(Error::missing());
            }
            Ok(json!({"ready":ready}))
        }
        PartyTransfer { target } => {
            let n=sqlx::query("UPDATE parties p SET leader=$2 WHERE leader=$1 AND closed_at IS NULL AND EXISTS(SELECT 1 FROM party_members m WHERE m.party_id=p.id AND m.account_id=$2)").bind(me).bind(target).execute(db).await?.rows_affected();
            if n == 0 {
                return Err(Error::forbidden());
            }
            Ok(json!({"transferred":true}))
        }
        PartyLeave => {
            let row=sqlx::query("SELECT p.id,p.leader,p.room_id FROM parties p JOIN party_members m ON m.party_id=p.id WHERE m.account_id=$1 FOR UPDATE OF p").bind(me).fetch_optional(&mut *db).await?.ok_or_else(Error::missing)?;
            let id: Uuid = row.get("id");
            let room: Uuid = row.get("room_id");
            let next:Option<Uuid>=sqlx::query_scalar("SELECT account_id FROM party_members WHERE party_id=$1 AND account_id<>$2 ORDER BY account_id LIMIT 1").bind(id).bind(me).fetch_optional(&mut *db).await?;
            if row.get::<Uuid, _>("leader") == me {
                match next {
                    Some(n) => {
                        sqlx::query("UPDATE parties SET leader=$2 WHERE id=$1")
                            .bind(id)
                            .bind(n)
                            .execute(&mut *db)
                            .await?;
                    }
                    None => {
                        sqlx::query("UPDATE parties SET closed_at=now() WHERE id=$1")
                            .bind(id)
                            .execute(&mut *db)
                            .await?;
                        sqlx::query("UPDATE rooms SET archived_at=now() WHERE id=$1")
                            .bind(room)
                            .execute(&mut *db)
                            .await?;
                    }
                }
            }
            sqlx::query("DELETE FROM party_members WHERE account_id=$1")
                .bind(me)
                .execute(&mut *db)
                .await?;
            sqlx::query("DELETE FROM room_members WHERE room_id=$1 AND account_id=$2")
                .bind(room)
                .bind(me)
                .execute(db)
                .await?;
            Ok(json!({"left":true}))
        }
        CommunityCreate { name } => {
            let id = Uuid::new_v4();
            sqlx::query("INSERT INTO communities(id,name,owner) VALUES($1,$2,$3)")
                .bind(id)
                .bind(label(name, 64)?)
                .bind(me)
                .execute(&mut *db)
                .await?;
            sqlx::query("INSERT INTO community_members(community_id,account_id,administrator) VALUES($1,$2,true)").bind(id).bind(me).execute(db).await?;
            Ok(json!({"community_id":id}))
        }
        Invite {
            kind,
            resource,
            target,
        } => invite(db, me, kind, *resource, *target).await,
        InviteRespond { id, accept } => respond(db, me, *id, *accept).await,
        Block { target, blocked } => {
            if *target == me {
                return Err(Error::invalid("自分はブロックできません。"));
            }
            if *blocked {
                sqlx::query(
                    "INSERT INTO blocks(actor,target) VALUES($1,$2) ON CONFLICT DO NOTHING",
                )
                .bind(me)
                .bind(target)
                .execute(&mut *db)
                .await?;
                sqlx::query("DELETE FROM friendships WHERE first_id=least($1,$2) AND second_id=greatest($1,$2)").bind(me).bind(target).execute(&mut *db).await?;
                sqlx::query("UPDATE invitations SET state='cancelled' WHERE state='pending' AND ((sender=$1 AND recipient=$2) OR (sender=$2 AND recipient=$1))").bind(me).bind(target).execute(&mut *db).await?;
            } else {
                sqlx::query("DELETE FROM blocks WHERE actor=$1 AND target=$2")
                    .bind(me)
                    .bind(target)
                    .execute(&mut *db)
                    .await?;
            }
            Ok(json!({"blocked":blocked}))
        }
        Privacy {
            display_name,
            dm_policy,
            activity_policy,
        } => {
            for policy in [dm_policy, activity_policy] {
                if !matches!(policy.as_str(), "friends" | "everyone" | "none") {
                    return Err(Error::invalid("公開範囲が不正です。"));
                }
            }
            sqlx::query("UPDATE principals SET name=$2 WHERE id=$1")
                .bind(me)
                .bind(label(display_name, 64)?)
                .execute(&mut *db)
                .await?;
            sqlx::query("UPDATE accounts SET dm_policy=$2,activity_policy=$3 WHERE id=$1")
                .bind(me)
                .bind(dm_policy)
                .bind(activity_policy)
                .execute(db)
                .await?;
            Ok(json!({"updated":true}))
        }
        NotificationsRead { through } => {
            sqlx::query("UPDATE notifications SET read_at=now() WHERE account_id=$1 AND id<=$2 AND read_at IS NULL").bind(me).bind(through).execute(db).await?;
            Ok(json!({"read":true}))
        }
        Report {
            target,
            reason,
            message_ids,
        } => {
            let evidence = crate::queries::evidence(db, me, message_ids).await?;
            let id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO reports(id,reporter,target,reason,evidence) VALUES($1,$2,$3,$4,$5)",
            )
            .bind(id)
            .bind(me)
            .bind(target)
            .bind(label(reason, 4000)?)
            .bind(evidence)
            .execute(db)
            .await?;
            Ok(json!({"report_id":id}))
        }
        ReportResolve {
            id,
            status,
            resolution,
        } => {
            if !actor.admin {
                return Err(Error::forbidden());
            }
            if !matches!(status.as_str(), "investigating" | "resolved" | "dismissed") {
                return Err(Error::invalid("通報の状態が不正です。"));
            }
            let n = sqlx::query("UPDATE reports SET status=$2,resolution=$3 WHERE id=$1")
                .bind(id)
                .bind(status)
                .bind(label(resolution, 4000)?)
                .execute(&mut *db)
                .await?
                .rows_affected();
            if n == 0 {
                return Err(Error::missing());
            }
            audit(db, me, "report.resolve", id, json!({"status":status})).await?;
            Ok(json!({"updated":true}))
        }
        Ban {
            target,
            hours,
            reason,
        } => {
            if !actor.admin || *target == me {
                return Err(Error::forbidden());
            }
            if !(0..=24 * 365 * 100).contains(hours) {
                return Err(Error::invalid("時間が範囲外です。"));
            }
            let reason = label(reason, 4000)?;
            sqlx::query("UPDATE accounts SET banned_until=CASE WHEN $2=0 THEN NULL ELSE now()+make_interval(hours=>$2) END WHERE id=$1 AND NOT administrator").bind(target).bind(hours).execute(&mut *db).await?;
            sqlx::query("DELETE FROM sessions WHERE account_id=$1")
                .bind(target)
                .execute(&mut *db)
                .await?;
            if *hours > 0 {
                crate::commands::job(
                    db,
                    me,
                    None,
                    "proxy",
                    "player.kick",
                    json!({"account_id":target,"reason":reason}),
                )
                .await?;
            }
            audit(
                db,
                me,
                "account.ban",
                target,
                json!({"hours":hours,"reason":reason}),
            )
            .await?;
            Ok(json!({"updated":true}))
        }
        RankSet { target, rank } => {
            if !actor.admin {
                return Err(Error::forbidden());
            }
            sqlx::query("UPDATE accounts SET trust_rank=$2 WHERE id=$1")
                .bind(target)
                .bind(rank)
                .execute(&mut *db)
                .await?;
            audit(db, me, "account.rank", target, json!({"rank":rank})).await?;
            Ok(json!({"updated":true}))
        }
        RankConfigure {
            id,
            name,
            server_count,
            concurrent_servers,
            memory_mib,
            cpu_millis,
            storage_mib,
        } => {
            if !actor.admin {
                return Err(Error::forbidden());
            }
            if *id < 0
                || *server_count < 0
                || *concurrent_servers < 0
                || concurrent_servers > server_count
                || *memory_mib < 0
                || *cpu_millis < 0
                || *storage_mib < 0
            {
                return Err(Error::invalid("上限が不正です。"));
            }
            sqlx::query("INSERT INTO trust_ranks VALUES($1,$2,$3,$4,$5,$6,$7) ON CONFLICT(id) DO UPDATE SET name=$2,server_count=$3,concurrent_servers=$4,memory_mib=$5,cpu_millis=$6,storage_mib=$7")
                .bind(id).bind(label(name,64)?).bind(server_count).bind(concurrent_servers).bind(memory_mib).bind(cpu_millis).bind(storage_mib).execute(&mut *db).await?;
            audit(db,me,"rank.configure",id,json!({"server_count":server_count,"concurrent_servers":concurrent_servers,"memory_mib":memory_mib,"cpu_millis":cpu_millis,"storage_mib":storage_mib})).await?;
            Ok(json!({"updated":true}))
        }
        _ => Err(Error::invalid("この操作は社交機能ではありません。")),
    }
}
