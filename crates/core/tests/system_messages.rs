use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = false)]
async fn generated_text_backfill_uses_provenance_and_preserves_custom_content(pool: PgPool) {
    let migrations = sqlx::migrate!("../../migrations");
    for migration in migrations.iter().filter(|migration| migration.version < 19) {
        sqlx::raw_sql(&migration.sql).execute(&pool).await.unwrap();
    }
    // These deliberately resemble old system text but have no system provenance.
    let player = Uuid::new_v4();
    sqlx::query("INSERT INTO principals(id,kind,name) VALUES($1,'player','取引手数料')")
        .bind(player)
        .execute(&pool)
        .await
        .unwrap();
    let asset = Uuid::new_v4();
    sqlx::query("INSERT INTO assets(id,owner,kind,title,state) VALUES($1,$2,'items','冒険準備の返却：エンダーアイ12個','escrowed')")
        .bind(asset).bind(player).execute(&pool).await.unwrap();
    sqlx::query(
        "UPDATE achievements SET title='私の実績',description='自由な説明' WHERE key='first_claim'",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("UPDATE trust_ranks SET name='独自のランク' WHERE id=0")
        .execute(&pool)
        .await
        .unwrap();
    let migration = migrations
        .iter()
        .find(|migration| migration.version == 19)
        .unwrap();
    sqlx::raw_sql(&migration.sql).execute(&pool).await.unwrap();
    let unchanged: (String, Option<Value>) =
        sqlx::query_as("SELECT title,title_message FROM assets WHERE id=$1")
            .bind(asset)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(unchanged, ("冒険準備の返却：エンダーアイ12個".into(), None));
    let custom: (String, String, Option<Value>, Option<Value>) = sqlx::query_as("SELECT title,description,title_message,description_message FROM achievements WHERE key='first_claim'")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(custom, ("私の実績".into(), "自由な説明".into(), None, None));
    let remaining: i64 = sqlx::query_scalar("SELECT count(*) FROM achievements WHERE title_message IS NOT NULL AND description_message IS NOT NULL")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(remaining, 6);
    let rank: (String, Option<Value>) =
        sqlx::query_as("SELECT name,name_message FROM trust_ranks WHERE id=0")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(rank, ("独自のランク".into(), None));
    let user_name: (String, Option<Value>) =
        sqlx::query_as("SELECT name,name_message FROM principals WHERE id=$1")
            .bind(player)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(user_name, ("取引手数料".into(), None));
    let system: Value = sqlx::query_scalar(
        "SELECT name_message FROM principals WHERE id='00000000-0000-0000-0000-000000000001'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(system["id"], "system.transaction_fees");
}
