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
    sqlx::query("INSERT INTO server_members(server_id,account_id,role) VALUES($1,$2,'member')").bind(server).bind(viewer.id).execute(&app.db).await.unwrap();
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
    sqlx::query("INSERT INTO server_members(server_id,account_id,role) VALUES($1,$2,'member')").bind(private).bind(viewer.id).execute(&app.db).await.unwrap();
    let (_,play)=http(&app,&viewer,"GET","/api/v1/view/play",json!({}),false).await;
    assert_eq!(play["play"]["preferred_server_id"],private.to_string());
    assert!(play["servers"].as_array().unwrap().iter().all(|s|s["status"].is_object()));
}
