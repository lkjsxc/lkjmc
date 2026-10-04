#[sqlx::test(migrations = "../../migrations")]
async fn people_are_friendships_and_presence_respects_privacy_and_access(pool: PgPool) {
    let app = app(pool);
    let viewer = account(&app, "Viewer", false).await;
    let friend = account(&app, "Friend", false).await;
    let pending = account(&app, "Pending", false).await;
    let (server, _) = official(&app).await;
    for (person, state) in [(&friend,"accepted"),(&pending,"pending")] {
        sqlx::query("INSERT INTO friendships(first_id,second_id,requester,state) VALUES(least($1,$2),greatest($1,$2),$2,$3)")
            .bind(viewer.id).bind(person.id).bind(state).execute(&app.db).await.unwrap();
        sqlx::query("INSERT INTO game_sessions(account_id,profile_id,native_uuid,session_id,server_id,lease_until) SELECT $1,id,$2,$3,$4,now()+interval '5 minutes' FROM profiles WHERE account_id=$1 AND status='active'")
            .bind(person.id).bind(Uuid::new_v4()).bind(Uuid::new_v4()).bind(server).execute(&app.db).await.unwrap();
    }
    let (status, people) = http(&app,&viewer,"GET","/api/v1/view/social?section=friends",json!({}),false).await;
    assert_eq!(status,StatusCode::OK,"{people}");
    let rows=people["friends"].as_array().unwrap();
    assert_eq!(rows.len(),2);
    assert_eq!(rows.iter().find(|f|f["id"]==friend.id.to_string()).unwrap()["server_id"],server.to_string());
    assert!(rows.iter().find(|f|f["id"]==pending.id.to_string()).unwrap()["server_id"].is_null());

    sqlx::query("UPDATE accounts SET activity_policy='none' WHERE id=$1").bind(friend.id).execute(&app.db).await.unwrap();
    assert!(lkjmc_core::player_views::friends(&app,viewer.id).await.unwrap().iter().all(|f|f.server_id.is_none()));
    sqlx::query("UPDATE accounts SET activity_policy='friends' WHERE id=$1").bind(friend.id).execute(&app.db).await.unwrap();
    sqlx::query("UPDATE servers SET visibility='private' WHERE id=$1").bind(server).execute(&app.db).await.unwrap();
    assert!(lkjmc_core::player_views::friends(&app,viewer.id).await.unwrap().iter().all(|f|f.server_id.is_none()));
    sqlx::query("INSERT INTO server_members(server_id,account_id,role) VALUES($1,$2,'guest')").bind(server).bind(viewer.id).execute(&app.db).await.unwrap();
    assert_eq!(lkjmc_core::player_views::friends(&app,viewer.id).await.unwrap().iter().find(|f|f.id==friend.id).unwrap().server_id,Some(server));
    sqlx::query("INSERT INTO blocks(actor,target) VALUES($1,$2)").bind(friend.id).bind(viewer.id).execute(&app.db).await.unwrap();
    assert!(!lkjmc_core::player_views::friends(&app,viewer.id).await.unwrap().iter().any(|f|f.id==friend.id));
}

#[sqlx::test(migrations = "../../migrations")]
async fn play_resume_never_restores_a_revoked_destination(pool: PgPool) {
    let app=app(pool);
    let viewer=account(&app,"Player",false).await;
    let stranger=account(&app,"Owner",false).await;
    let (official,_) = official(&app).await;
    let private=Uuid::new_v4();
    sqlx::query("INSERT INTO servers(id,owner,name,kind,visibility,version,software,memory_mib,cpu_millis,storage_mib) VALUES($1,$2,'Private','custom','private','test','paper',2048,2000,16384)")
        .bind(private).bind(stranger.id).execute(&app.db).await.unwrap();
    sqlx::query("INSERT INTO jobs(id,actor,server_id,worker,kind,state,payload) VALUES($1,$2,$3,'proxy','player.join','succeeded','{}')")
        .bind(Uuid::new_v4()).bind(viewer.id).bind(private).execute(&app.db).await.unwrap();
    let (status,play)=http(&app,&viewer,"GET","/api/v1/view/play",json!({}),false).await;
    assert_eq!(status,StatusCode::OK,"{play}");
    assert_eq!(play["play"]["preferred_server_id"],official.to_string());
    assert_eq!(play["play"]["identity_ready"],false);
    assert!(!play["servers"].as_array().unwrap().iter().any(|s|s["id"]==private.to_string()));
    sqlx::query("INSERT INTO server_members(server_id,account_id,role) VALUES($1,$2,'guest')").bind(private).bind(viewer.id).execute(&app.db).await.unwrap();
    let (_,play)=http(&app,&viewer,"GET","/api/v1/view/play",json!({}),false).await;
    assert_eq!(play["play"]["preferred_server_id"],private.to_string());
    assert!(play["servers"].as_array().unwrap().iter().all(|s|s["status"].is_object()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn world_detail_does_not_hold_a_connection_while_waiting_for_another(pool: PgPool) {
    let single = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(1))
        .connect_with((*pool.connect_options()).clone()).await.unwrap();
    let app = app(single);
    let viewer = account(&app, "Reader", true).await;
    let (server, _) = official(&app).await;
    for section in ["overview", "homes", "manage-overview", "manage-members"] {
        let (status, page) = http(&app, &viewer, "GET", &format!("/api/v1/servers/{server}?section={section}"), json!({}), false).await;
        assert_eq!(status, StatusCode::OK, "{section}: {page}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn player_worlds_never_present_missing_stale_or_future_observations_as_ready(pool: PgPool) {
    let app = app(pool);
    let viewer = account(&app, "Observer", false).await;
    let (server, _) = official(&app).await;
    for observed_at in [None, Some(chrono::Utc::now() - chrono::Duration::minutes(2)), Some(chrono::Utc::now() + chrono::Duration::minutes(2))] {
        sqlx::query("UPDATE servers SET observed='running',last_observed_at=$2 WHERE id=$1")
            .bind(server).bind(observed_at).execute(&app.db).await.unwrap();
        for path in ["/api/v1/view/play".to_string(), format!("/api/v1/servers/{server}")] {
            let (status, page) = http(&app, &viewer, "GET", &path, json!({}), false).await;
            assert_eq!(status, StatusCode::OK, "{page}");
            let world = page.get("server").unwrap_or_else(|| &page["servers"][0]);
            assert_eq!(world["observed"], "unknown");
            assert_eq!(world["status"]["game_state"], "unknown");
            assert_eq!(world["status"]["actions"]["join"]["allowed"], false);
            assert_eq!(world["status"]["actions"]["join"]["reason"], "observation_stale");
        }
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn expedition_journal_is_participant_scoped_and_pages_equal_timestamps(pool: PgPool) {
    let app=app(pool);
    let member=account(&app,"Explorer",false).await;
    let other=account(&app,"Other explorer",false).await;
    let administrator=account(&app,"Operator",true).await;
    let private=Uuid::new_v4();
    sqlx::query("INSERT INTO adventures(id,owner,state,created_at) VALUES($1,$2,'closed','2026-10-04T00:00:00Z')")
        .bind(private).bind(other.id).execute(&app.db).await.unwrap();
    for _ in 0..52 {
        sqlx::query("INSERT INTO adventures(id,owner,state,created_at) VALUES($1,$2,'closed','2026-10-04T00:00:00Z')")
            .bind(Uuid::new_v4()).bind(member.id).execute(&app.db).await.unwrap();
    }
    let mut path="/api/v1/expeditions".to_string();
    let mut seen=std::collections::BTreeSet::new();
    let mut first_cursor=None;
    loop {
        let (status,page)=http(&app,&member,"GET",&path,json!({}),false).await;
        assert_eq!(status,StatusCode::OK,"{page}");
        let rows=page["expeditions"].as_array().unwrap();
        assert!(rows.len()<=25);
        for row in rows {
            assert_ne!(row["id"],private.to_string());
            assert!(seen.insert(row["id"].as_str().unwrap().to_string()));
        }
        let Some(cursor)=page["next_cursor"].as_str() else {break};
        first_cursor.get_or_insert_with(||cursor.to_string());
        path=format!("/api/v1/expeditions?cursor={cursor}");
    }
    assert_eq!(seen.len(),52);
    for viewer in [&member,&administrator] {
        assert_eq!(http(&app,viewer,"GET",&format!("/api/v1/expeditions/{private}"),json!({}),false).await.0,StatusCode::NOT_FOUND);
    }
    assert_eq!(http(&app,&other,"GET",&format!("/api/v1/expeditions?cursor={}",first_cursor.unwrap()),json!({}),false).await.0,StatusCode::BAD_REQUEST);
    assert_eq!(http(&app,&member,"GET","/api/v1/expeditions?cursor=invalid",json!({}),false).await.0,StatusCode::BAD_REQUEST);
}
