use super::*;

#[derive(Default, Deserialize)]
pub struct TeamQuery {
    pub after: Option<Uuid>,
    pub limit: Option<i64>,
}

// Effective actor capabilities are separate from the stored member flags used
// by role editors. A leader or administrator can exercise all team capabilities.
const PERMISSIONS: &str = "jsonb_build_object('can_build',t.leader=$1 OR m.can_administer OR m.can_build,'can_sell',t.leader=$1 OR m.can_administer OR m.can_sell,'can_spend',t.leader=$1 OR m.can_administer OR m.can_spend,'can_manage_members',t.leader=$1 OR m.can_administer OR m.can_manage_members,'can_administer',t.leader=$1 OR m.can_administer)";

fn summary_sql() -> String {
    format!(
        "jsonb_build_object('id',t.id,'name',p.name,'leader',t.leader,'room_id',t.room_id,'permissions',{PERMISSIONS},'member_count',(SELECT count(*) FROM team_members tm WHERE tm.team_id=t.id),'is_contribution_team',s.team_id IS NOT NULL)"
    )
}

pub(super) async fn collection(app: &App, actor: Uuid) -> Result<Value> {
    let summary = summary_sql();
    Ok(sqlx::query_scalar(&format!(
        "SELECT jsonb_build_object('teams',(SELECT coalesce(jsonb_agg({summary} ORDER BY lower(p.name),t.id),'[]') FROM teams t JOIN team_members m ON m.team_id=t.id JOIN principals p ON p.id=t.id LEFT JOIN team_contribution_selection s ON s.account_id=m.account_id AND s.team_id=t.id WHERE m.account_id=$1 AND t.disbanded_at IS NULL),'contribution_team_id',(SELECT s.team_id FROM team_contribution_selection s JOIN teams t ON t.id=s.team_id WHERE s.account_id=$1 AND t.disbanded_at IS NULL))"
    )).bind(actor).fetch_one(&app.db).await?)
}

pub(super) async fn owners(app: &App, actor: Uuid) -> Result<Value> {
    Ok(sqlx::query_scalar(&format!(
        "SELECT coalesce(jsonb_agg(to_jsonb(p)||jsonb_build_object('wallet',to_jsonb(w),'land',to_jsonb(l),'used_chunks',(SELECT coalesce(sum(chunks),0) FROM claims c WHERE c.owner=p.id AND c.state<>'released'),'permissions',CASE WHEN p.id=$1 THEN jsonb_build_object('can_build',true,'can_sell',true,'can_spend',true,'can_manage_members',false,'can_administer',false) ELSE {PERMISSIONS} END) ORDER BY p.id<>$1,lower(p.name),p.id),'[]') FROM principals p JOIN wallets w ON w.owner=p.id JOIN land_allowances l ON l.owner=p.id LEFT JOIN teams t ON t.id=p.id LEFT JOIN team_members m ON m.team_id=t.id AND m.account_id=$1 WHERE p.id=$1 OR m.account_id=$1 AND t.disbanded_at IS NULL"
    )).bind(actor).fetch_one(&app.db).await?)
}

pub(super) async fn claims(app: &App, actor: Uuid) -> Result<Value> {
    Ok(sqlx::query_scalar(
        "SELECT coalesce(jsonb_agg(to_jsonb(c) ORDER BY c.name,c.id),'[]') FROM claims c WHERE c.state<>'released' AND (c.owner=$1 OR EXISTS(SELECT 1 FROM team_members m JOIN teams t ON t.id=m.team_id WHERE m.team_id=c.owner AND m.account_id=$1 AND t.disbanded_at IS NULL))"
    ).bind(actor).fetch_one(&app.db).await?)
}

pub(super) async fn achievements(app: &App, actor: Uuid) -> Result<Value> {
    Ok(sqlx::query_scalar(
        "WITH owners AS (SELECT p.id,p.name,p.kind FROM principals p WHERE p.id=$1 OR EXISTS(SELECT 1 FROM team_members m JOIN teams t ON t.id=m.team_id WHERE m.team_id=p.id AND m.account_id=$1 AND t.disbanded_at IS NULL)) SELECT coalesce(jsonb_agg(jsonb_build_object('owner',to_jsonb(o),'achievements',(SELECT coalesce(jsonb_agg(to_jsonb(a)||jsonb_build_object('progress',coalesce(p.progress,0),'earned_at',p.earned_at) ORDER BY a.key),'[]') FROM achievements a LEFT JOIN achievement_progress p ON p.achievement=a.key AND p.owner=o.id WHERE a.team=(o.kind='team'))) ORDER BY o.kind='team',lower(o.name),o.id),'[]') FROM owners o"
    ).bind(actor).fetch_one(&app.db).await?)
}

pub async fn detail(
    State(app): State<App>,
    actor: Actor,
    Path(id): Path<Uuid>,
    Query(query): Query<TeamQuery>,
) -> Result<Json<Value>> {
    let mut tx = app.db.begin().await?;
    let summary = summary_sql();
    // Keep authorization and the member page within the same membership. These
    // shared locks also prevent a disband between authorization and the read.
    let mut team: Value = sqlx::query_scalar(&format!(
        "SELECT {summary} FROM teams t JOIN team_members m ON m.team_id=t.id JOIN principals p ON p.id=t.id LEFT JOIN team_contribution_selection s ON s.account_id=m.account_id AND s.team_id=t.id WHERE m.account_id=$1 AND t.id=$2 AND t.disbanded_at IS NULL FOR SHARE OF t,m"
    )).bind(actor.id).bind(id).fetch_optional(&mut *tx).await?.ok_or_else(Error::forbidden)?;
    let limit = query.limit.unwrap_or(100).clamp(1, 100) as usize;
    let mut members: Vec<Value> = sqlx::query_scalar(
        "SELECT to_jsonb(m)||jsonb_build_object('name',p.name) FROM team_members m JOIN principals p ON p.id=m.account_id WHERE m.team_id=$1 AND ($2::uuid IS NULL OR m.account_id>$2) ORDER BY m.account_id LIMIT $3"
    ).bind(id).bind(query.after).bind(limit as i64 + 1).fetch_all(&mut *tx).await?;
    let next = if members.len() > limit {
        members.truncate(limit);
        members.last().and_then(|m| m.get("account_id")).cloned()
    } else {
        None
    };
    team["members"] = json!(members);
    team["members_next_after"] = json!(next);
    let contribution_team: Option<Uuid> = sqlx::query_scalar(
        "SELECT s.team_id FROM team_contribution_selection s JOIN teams t ON t.id=s.team_id WHERE s.account_id=$1 AND t.disbanded_at IS NULL"
    ).bind(actor.id).fetch_optional(&mut *tx).await?;
    tx.commit().await?;
    let mut result = json!({"team":team,"contribution_team_id":contribution_team});
    crate::system_message::project_system_content(&mut result);
    Ok(Json(result))
}
