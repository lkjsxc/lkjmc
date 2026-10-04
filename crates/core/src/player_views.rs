//! Player-facing projections. Visibility is checked before selecting a resume
//! destination or exposing another player's presence.
use crate::{
    App,
    auth::Actor,
    error::{Error, Result},
};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize, FromRow)]
pub struct Friend {
    pub id: Uuid,
    pub name: String,
    pub state: String,
    pub requester: Uuid,
    pub server_id: Option<Uuid>,
}

/// A pending request is not permission to observe someone's presence. Blocks
/// work in either direction, and private destination IDs remain private.
pub async fn friends(app: &App, account: Uuid) -> Result<Vec<Friend>> {
    Ok(sqlx::query_as::<_, Friend>(r#"
        SELECT p.id,p.name,f.state,f.requester,
          CASE WHEN f.state='accepted' AND a.activity_policy<>'none'
            AND (s.visibility='public' OR s.owner=$1
              OR EXISTS(SELECT 1 FROM accounts WHERE id=$1 AND administrator)
              OR EXISTS(SELECT 1 FROM server_members WHERE server_id=s.id AND account_id=$1)
              OR EXISTS(SELECT 1 FROM community_members WHERE community_id=s.community_id AND account_id=$1))
          THEN g.server_id ELSE NULL END AS server_id
        FROM friendships f
        JOIN principals p ON p.id=CASE WHEN f.first_id=$1 THEN f.second_id ELSE f.first_id END
        JOIN accounts a ON a.id=p.id AND a.merged_into IS NULL
        LEFT JOIN game_sessions g ON g.account_id=p.id AND g.lease_until>now()
        LEFT JOIN servers s ON s.id=g.server_id
        WHERE $1 IN (f.first_id,f.second_id)
          AND NOT EXISTS(SELECT 1 FROM blocks b WHERE
            (b.actor=$1 AND b.target=p.id) OR (b.actor=p.id AND b.target=$1))
        ORDER BY f.state,p.name,p.id
    "#).bind(account).fetch_all(&app.db).await?)
}

#[derive(Serialize, FromRow)]
pub struct PlaySession {
    pub server_id: Option<Uuid>,
    pub client: String,
}

#[derive(Serialize)]
pub struct PlayContext {
    pub preferred_server_id: Option<Uuid>,
    pub game_session: Option<PlaySession>,
    pub identity_ready: bool,
}

pub async fn servers(app: &App, actor: &Actor, managed: bool) -> Result<Vec<Value>> {
    let mut servers: Vec<Value> = sqlx::query_scalar(r#"
      SELECT jsonb_build_object(
        'id',s.id,'name',s.name,'kind',s.kind,'visibility',s.visibility,
        'version',s.version,'software',s.software,'capabilities',s.capabilities,
        'desired',s.desired,'observed',s.observed,'players',s.players,
        'last_observed_at',s.last_observed_at,'maintenance',s.maintenance,
        'machine_observed',s.machine_observed,'machine_observed_at',s.machine_observed_at,
        'can_manage',(s.owner=$1 OR EXISTS(SELECT 1 FROM accounts WHERE id=$1 AND administrator)
          OR EXISTS(SELECT 1 FROM server_members WHERE server_id=s.id AND account_id=$1 AND role IN ('operator','administrator'))
          OR EXISTS(SELECT 1 FROM community_members WHERE community_id=s.community_id AND account_id=$1 AND administrator)),
        'can_administer',(s.owner=$1 OR EXISTS(SELECT 1 FROM accounts WHERE id=$1 AND administrator)
          OR EXISTS(SELECT 1 FROM server_members WHERE server_id=s.id AND account_id=$1 AND role='administrator')
          OR EXISTS(SELECT 1 FROM community_members WHERE community_id=s.community_id AND account_id=$1 AND administrator)),
        'active_operation',(SELECT jsonb_build_object('id',j.id,'kind',j.kind,'state',j.state,'progress',jsonb_build_object('phase',j.progress->'phase'),'can_inspect',(j.actor=$1 OR EXISTS(SELECT 1 FROM accounts WHERE id=$1 AND administrator)))
          FROM jobs j WHERE j.server_id=s.id AND j.worker='host' AND j.state IN ('queued','leased','waiting')
            AND j.kind NOT IN ('server.logs','server.files','server.file.read') ORDER BY j.created_at,j.id LIMIT 1))
        || CASE WHEN $2 THEN jsonb_build_object('memory_mib',s.memory_mib,'cpu_millis',s.cpu_millis,'storage_mib',s.storage_mib,'inspection',s.inspection) ELSE '{}'::jsonb END
      FROM servers s WHERE s.visibility='public' OR s.owner=$1
        OR EXISTS(SELECT 1 FROM accounts WHERE id=$1 AND administrator)
        OR EXISTS(SELECT 1 FROM server_members WHERE server_id=s.id AND account_id=$1)
        OR EXISTS(SELECT 1 FROM community_members WHERE community_id=s.community_id AND account_id=$1)
      ORDER BY CASE s.kind WHEN 'official' THEN 0 WHEN 'custom' THEN 1 ELSE 2 END,s.name,s.id
    "#).bind(actor.id).bind(managed).fetch_all(&app.db).await?;
    if managed {
        servers.retain(|server| server["can_manage"] == true);
    }
    for server in &mut servers {
        server["status"] = serde_json::to_value(crate::server_tools::server_status(server))
            .map_err(Error::internal)?;
    }
    Ok(servers)
}

pub async fn play(app: &App, actor: &Actor) -> Result<Value> {
    let servers = servers(app, actor, false).await?;
    let allowed = |id: Uuid| {
        servers
            .iter()
            .any(|s| s["kind"] != "lobby" && s["id"].as_str() == Some(&id.to_string()))
    };
    let mut context = context(app, actor).await?;
    let current = context
        .game_session
        .as_ref()
        .and_then(|s| s.server_id)
        .filter(|id| allowed(*id));
    let recent: Vec<Uuid> = sqlx::query_scalar(
        "SELECT server_id FROM jobs WHERE actor=$1 AND kind='player.join' AND state='succeeded' AND server_id IS NOT NULL ORDER BY updated_at DESC,id DESC LIMIT 25"
    ).bind(actor.id).fetch_all(&app.db).await?;
    let preferred_server_id = current
        .or_else(|| recent.into_iter().find(|id| allowed(*id)))
        .or_else(|| {
            servers
                .iter()
                .find(|s| s["kind"] != "lobby")
                .and_then(|s| s["id"].as_str())
                .and_then(|s| s.parse().ok())
        });
    let invitations: Vec<Value> = sqlx::query_scalar(
        "SELECT to_jsonb(i)||jsonb_build_object('sender_name',p.name) FROM invitations i JOIN principals p ON p.id=i.sender WHERE i.recipient=$1 AND i.state='pending' AND i.expires_at>now() ORDER BY i.created_at DESC,i.id DESC LIMIT 3"
    ).bind(actor.id).fetch_all(&app.db).await?;
    let friends: Vec<Friend> = friends(app, actor.id)
        .await?
        .into_iter()
        .filter(|friend| friend.state == "accepted" && friend.server_id.is_some())
        .take(12)
        .collect();
    context.preferred_server_id = preferred_server_id;
    Ok(json!({"servers":servers,"play":context,"invitations":invitations,"friends":friends}))
}

pub async fn context(app: &App, actor: &Actor) -> Result<PlayContext> {
    let game_session = sqlx::query_as::<_, PlaySession>(
        "SELECT server_id,client FROM game_sessions WHERE account_id=$1 AND lease_until>now()",
    )
    .bind(actor.id)
    .fetch_optional(&app.db)
    .await?;
    let identity_ready = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM identities WHERE account_id=$1 AND issuer IN ('java','bedrock'))"
    ).bind(actor.id).fetch_one(&app.db).await?;
    Ok(PlayContext {
        preferred_server_id: None,
        game_session,
        identity_ready,
    })
}
