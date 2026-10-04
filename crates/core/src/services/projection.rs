use super::*;

pub async fn projection(State(app): State<App>, service: Service) -> Result<Json<Value>> {
    let mut result = json!({"role":service.role,"development":app.config.development});
    if matches!(service.role.as_str(), "host" | "proxy") {
        result["servers"] = sqlx::query_scalar::<_, Value>(
            "SELECT coalesce(jsonb_agg(to_jsonb(s)),'[]') FROM servers s",
        )
        .fetch_one(&app.db)
        .await?;
    }
    if matches!(service.role.as_str(), "proxy" | "official" | "lobby") {
        result["sessions"]=sqlx::query_scalar::<_,Value>("SELECT coalesce(jsonb_agg(jsonb_build_object('account_id',g.account_id,'native_uuid',g.native_uuid,'profile_id',g.profile_id,'session_id',g.session_id,'server_id',g.server_id,'combat_until',g.combat_until,'name',p.name,'language',a.language)),'[]') FROM game_sessions g JOIN accounts a ON a.id=g.account_id JOIN principals p ON p.id=g.account_id WHERE g.lease_until>now() AND ($1::text='proxy' OR g.server_id=$2)").bind(&service.role).bind(service.server_id).fetch_one(&app.db).await?;
    }
    if matches!(service.role.as_str(), "official" | "lobby") {
        result["worlds"] = sqlx::query_scalar::<_, Value>(
            "SELECT coalesce(jsonb_agg(to_jsonb(w)),'[]') FROM worlds w WHERE server_id=$1",
        )
        .bind(service.server_id)
        .fetch_one(&app.db)
        .await?;
    }
    if service.role == "official" {
        result["claims"]=sqlx::query_scalar::<_,Value>("SELECT coalesce(jsonb_agg(to_jsonb(c)||jsonb_build_object('native_uuid',(SELECT native_uuid FROM profiles WHERE account_id=c.owner AND status='active'),'members',(SELECT coalesce(jsonb_agg(jsonb_build_object('account_id',m.account_id,'native_uuid',p.native_uuid,'can_build',m.can_build OR m.can_administer OR m.account_id=t.leader)),'[]') FROM team_members m JOIN teams t ON t.id=m.team_id LEFT JOIN profiles p ON p.account_id=m.account_id AND p.status='active' WHERE m.team_id=c.owner))),'[]') FROM claims c JOIN worlds w ON w.id=c.world_id WHERE w.server_id=$1 AND c.state<>'released'").bind(service.server_id).fetch_one(&app.db).await?;
        result["assets"]=sqlx::query_scalar::<_,Value>("SELECT coalesce(jsonb_agg(to_jsonb(a)),'[]') FROM assets a JOIN jobs j ON j.id=a.job_id WHERE j.server_id=$1 AND (a.state IN ('capturing','placing','quarantined') OR a.locked_claim_id IS NOT NULL)")
            .bind(service.server_id).fetch_one(&app.db).await?;
        result["native_owners"]=sqlx::query_scalar::<_,Value>("SELECT coalesce(jsonb_agg(jsonb_build_object('native_uuid',p.native_uuid,'account_id',p.account_id,'status',p.status)),'[]') FROM profiles p WHERE native_uuid IS NOT NULL").fetch_one(&app.db).await?;
        result["adventures"]=sqlx::query_scalar::<_,Value>("SELECT coalesce(jsonb_agg(to_jsonb(a)||jsonb_build_object('participants',(SELECT coalesce(jsonb_agg(jsonb_build_object('account_id',ap.account_id,'committed',ap.committed_at IS NOT NULL,'consented_at',ap.consented_at,'committed_at',ap.committed_at)),'[]') FROM adventure_participants ap WHERE ap.adventure_id=a.id AND ap.released_at IS NULL))),'[]') FROM adventures a WHERE state NOT IN ('closed','refunded')").fetch_one(&app.db).await?;
        result["consents"]=sqlx::query_scalar::<_,Value>("SELECT coalesce(jsonb_agg(to_jsonb(c)),'[]') FROM asset_consents c JOIN assets a ON a.id=c.asset_id WHERE a.state='capturing'").fetch_one(&app.db).await?;
        result["paused"] = sqlx::query_scalar::<_, Value>(
            "SELECT value FROM settings WHERE key='official_mutations_paused'",
        )
        .fetch_one(&app.db)
        .await?;
    }
    Ok(Json(result))
}
