async fn timeline_room(app: &App, owner: &Actor, members: &[&Actor]) -> Uuid {
    let room = Uuid::new_v4();
    sqlx::query("INSERT INTO rooms(id,kind,name,owner) VALUES($1,'group','Test conversation',$2)")
        .bind(room).bind(owner.id).execute(&app.db).await.unwrap();
    for member in members {
        sqlx::query("INSERT INTO room_members(room_id,account_id) VALUES($1,$2)")
            .bind(room).bind(member.id).execute(&app.db).await.unwrap();
    }
    room
}
async fn timeline_message(app: &App, room: Uuid, author: Uuid, body: &str) -> i64 {
    sqlx::query_scalar("INSERT INTO messages(room_id,author,body,created_at) VALUES($1,$2,$3,'2026-10-03T00:00:00Z') RETURNING id")
        .bind(room).bind(author).bind(body).fetch_one(&app.db).await.unwrap()
}
async fn timeline_job(app: &App, actor: Uuid, kind: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO jobs(id,actor,worker,kind,payload,state,created_at) VALUES($1,$2,'host',$3,'{}','queued','2026-10-03T00:00:00Z')")
        .bind(id).bind(actor).bind(kind).execute(&app.db).await.unwrap();
    id
}

#[sqlx::test(migrations = "../../migrations")]
async fn timeline_combines_only_current_private_membership_and_own_events(pool: PgPool) {
    let app = app(pool);
    let a = account(&app, "Alice", true).await;
    let b = account(&app, "Bob", false).await;
    let c = account(&app, "Carol", false).await;
    let shared = timeline_room(&app, &a, &[&a, &b]).await;
    let private = timeline_room(&app, &b, &[&b, &c]).await;
    let message = timeline_message(&app, shared, b.id, "Hello Alice").await;
    timeline_message(&app, private, c.id, "Not for Alice, even as administrator").await;
    let job = timeline_job(&app, a.id, "server.start").await;
    timeline_job(&app, b.id, "server.start").await;
    for kind in ["server.logs", "server.files", "server.file.read"] {
        timeline_job(&app, a.id, kind).await;
    }
    for (actor, kind) in [(a.id,"achievement"),(a.id,"message"),(a.id,"job_finished"),(b.id,"achievement")] {
        sqlx::query("INSERT INTO notifications(account_id,kind,body) VALUES($1,$2,'{}')")
            .bind(actor).bind(kind).execute(&app.db).await.unwrap();
    }
    let (status, value) = http(&app, &a, "GET", "/api/v1/timeline", json!({}), false).await;
    assert_eq!(status, StatusCode::OK, "{value}");
    let items = value["items"].as_array().unwrap();
    assert_eq!(items.len(), 3, "{value}");
    assert!(items.iter().any(|v| v["message_id"] == message && v["body"] == "Hello Alice"));
    assert!(items.iter().any(|v| v["job_id"] == job.to_string()));
    assert!(items.iter().any(|v| v["type"] == "notification" && v["kind"] == "achievement"));
    assert_eq!(value["rooms"].as_array().unwrap().len(), 1);
    assert_eq!(value["rooms"][0]["id"], shared.to_string());
    assert!(!value.to_string().contains("Not for Alice"));
    let (status, value) = http(&app, &a, "GET", &format!("/api/v1/timeline?room={shared}"), json!({}), false).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["items"].as_array().unwrap().len(), 1);
    assert_eq!(http(&app, &a, "GET", &format!("/api/v1/timeline?room={private}"), json!({}), false).await.0, StatusCode::FORBIDDEN);
    let (_, value) = http(&app, &a, "GET", "/api/v1/timeline?kind=events", json!({}), false).await;
    assert_eq!(value["items"].as_array().unwrap().len(), 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn timeline_cursor_is_bounded_stable_and_scope_bound_at_equal_times(pool: PgPool) {
    let app = app(pool);
    let a = account(&app, "Reader", false).await;
    let room = timeline_room(&app, &a, &[&a]).await;
    for n in 0..63 {
        timeline_message(&app, room, a.id, &format!("Message {n}")).await;
    }
    for n in 0..15 {
        timeline_job(&app, a.id, "server.start").await;
        sqlx::query("INSERT INTO notifications(account_id,kind,body,created_at) VALUES($1,'achievement',$2,'2026-10-03T00:00:00Z')")
            .bind(a.id).bind(json!({"index":n})).execute(&app.db).await.unwrap();
    }
    let mut path = "/api/v1/timeline".to_string();
    let mut seen = std::collections::BTreeSet::new();
    let mut first_cursor = None;
    loop {
        let (status, page) = http(&app, &a, "GET", &path, json!({}), false).await;
        assert_eq!(status, StatusCode::OK, "{page}");
        let items = page["items"].as_array().unwrap();
        assert!(!items.is_empty() && items.len() <= 50);
        let mut previous = String::new();
        for item in items {
            let key = item["id"].as_str().unwrap().to_string();
            assert!(key > previous);
            assert!(seen.insert(key.clone()), "duplicate {key}");
            previous = key;
        }
        let Some(cursor) = page["next_cursor"].as_str() else { break };
        first_cursor.get_or_insert(cursor.to_string());
        path = format!("/api/v1/timeline?before={cursor}");
    }
    assert_eq!(seen.len(), 93);
    let cursor = first_cursor.unwrap();
    assert_eq!(http(&app, &a, "GET", &format!("/api/v1/timeline?kind=messages&before={cursor}"), json!({}), false).await.0, StatusCode::BAD_REQUEST);
    let b = account(&app, "Other reader", false).await;
    assert_eq!(http(&app, &b, "GET", &format!("/api/v1/timeline?before={cursor}"), json!({}), false).await.0, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn timeline_refresh_revalidates_old_items_and_returns_deletion_tombstones(pool: PgPool) {
    let app = app(pool);
    let a = account(&app, "Reader", false).await;
    let b = account(&app, "Sender", false).await;
    let shared = timeline_room(&app, &a, &[&a, &b]).await;
    let private = timeline_room(&app, &b, &[&b]).await;
    let old = timeline_message(&app, shared, b.id, "Old visible message").await;
    let secret = timeline_message(&app, private, b.id, "Hidden message").await;
    let job = timeline_job(&app, a.id, "server.start").await;
    sqlx::query("UPDATE jobs SET created_at='2000-01-01T00:00:00Z',state='succeeded' WHERE id=$1").bind(job).execute(&app.db).await.unwrap();
    sqlx::query("UPDATE messages SET created_at='2000-01-01T00:00:00Z',deleted_at=now() WHERE id=$1").bind(old).execute(&app.db).await.unwrap();
    for _ in 0..51 { timeline_message(&app, shared, a.id, "Recent message").await; }
    let path = format!("/api/v1/timeline?known=message:{old},message:{secret},job:{job}");
    let (status, page) = http(&app, &a, "GET", &path, json!({}), false).await;
    assert_eq!(status, StatusCode::OK, "{page}");
    assert_eq!(page["items"].as_array().unwrap().len(), 50);
    let updates = page["updates"].as_array().unwrap();
    let tombstone = updates.iter().find(|v| v["message_id"] == old).unwrap();
    assert_eq!(tombstone["body"], "");
    assert!(!tombstone["deleted_at"].is_null());
    assert!(updates.iter().any(|v| v["job_id"] == job.to_string() && v["state"] == "succeeded"));
    assert_eq!(page["removed_ids"], json!([format!("message:{secret}")]));
    sqlx::query("INSERT INTO blocks(actor,target) VALUES($1,$2)").bind(b.id).bind(a.id).execute(&app.db).await.unwrap();
    let (_, page) = http(&app, &a, "GET", &path, json!({}), false).await;
    assert!(page["removed_ids"].as_array().unwrap().contains(&json!(format!("message:{old}"))));
    sqlx::query("DELETE FROM room_members WHERE room_id=$1 AND account_id=$2").bind(shared).bind(a.id).execute(&app.db).await.unwrap();
    let (_, page) = http(&app, &a, "GET", &path, json!({}), false).await;
    assert!(page["rooms"].as_array().unwrap().is_empty());
    assert!(!page["items"].as_array().unwrap().iter().any(|v| v["type"] == "message"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn timeline_rejects_invalid_or_unbounded_inputs_without_private_payload(pool: PgPool) {
    let app = app(pool);
    let a = account(&app, "Reader", false).await;
    for query in ["kind=everything", "before=bad", "known=message:-1", "known=job:no", "known=other:1", "room=broken"] {
        let status = http(&app, &a, "GET", &format!("/api/v1/timeline?{query}"), json!({}), false).await.0;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{query}");
    }
    let known = (1..=201).map(|n| format!("message:{n}")).collect::<Vec<_>>().join(",");
    assert_eq!(http(&app, &a, "GET", &format!("/api/v1/timeline?known={known}"), json!({}), false).await.0, StatusCode::BAD_REQUEST);
}
