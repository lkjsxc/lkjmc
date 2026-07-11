const TEST_DATABASE_LOCK: i64 = 752647;

pub(crate) struct TestDatabase(postgres::Client);

impl TestDatabase {
    pub(crate) fn client_mut(&mut self) -> &mut postgres::Client {
        &mut self.0
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        let _ = self
            .0
            .execute("select pg_advisory_unlock($1)", &[&TEST_DATABASE_LOCK]);
    }
}

pub(crate) fn migrate(database_url: &str) -> Result<TestDatabase, String> {
    let mut client =
        lkjmc_store::pool::connect_single(database_url).map_err(|error| error.to_string())?;
    client
        .batch_execute("set lock_timeout = 0; set statement_timeout = 0")
        .map_err(|error| error.to_string())?;
    client
        .execute("select pg_advisory_lock($1)", &[&TEST_DATABASE_LOCK])
        .map_err(|error| error.to_string())?;
    lkjmc_store::migrate::apply(&mut client).map_err(|error| error.to_string())?;
    Ok(TestDatabase(client))
}

pub(crate) fn try_lock(client: &mut postgres::Client) -> Result<bool, String> {
    client
        .query_one("select pg_try_advisory_lock($1)", &[&TEST_DATABASE_LOCK])
        .map(|row| row.get(0))
        .map_err(|error| error.to_string())
}

pub(crate) fn reset_and_migrate(database_url: &str) -> Result<TestDatabase, String> {
    let mut database = migrate(database_url)?;
    database
        .client_mut()
        .batch_execute("drop schema public cascade; create schema public")
        .map_err(|error| error.to_string())?;
    lkjmc_store::migrate::apply(database.client_mut()).map_err(|error| error.to_string())?;
    Ok(database)
}
