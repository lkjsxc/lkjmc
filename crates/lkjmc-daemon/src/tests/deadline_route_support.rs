use std::time::Duration;

pub(super) fn lock_route(transaction: &mut postgres::Transaction<'_>) -> Result<(), String> {
    transaction
        .query_one(
            "select revision from daemon_token_revision where singleton = true for update",
            &[],
        )
        .map(|_| ())
        .map_err(|error| error.to_string())
}

pub(super) fn waiting_backend(
    transaction: &mut postgres::Transaction<'_>,
    application_name: &str,
) -> Result<i32, String> {
    let query = "select min(activity.pid)::int
                 from pg_stat_activity activity
                 join pg_locks lock on lock.pid = activity.pid
                 where activity.application_name = $1 and not lock.granted";
    for _ in 0..200 {
        let row = transaction
            .query_one(query, &[&application_name])
            .map_err(|error| error.to_string())?;
        if let Some(pid) = row.get::<_, Option<i32>>(0) {
            return Ok(pid);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Err("route database query did not reach its PostgreSQL lock".into())
}
