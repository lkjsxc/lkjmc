use uuid::Uuid;

pub(crate) struct TestDatabase {
    client: postgres::Client,
    database_url: String,
    schema: String,
}

impl TestDatabase {
    pub(crate) fn client_mut(&mut self) -> &mut postgres::Client {
        &mut self.client
    }

    pub(crate) fn url(&self) -> &str {
        &self.database_url
    }

    pub(crate) fn schema(&self) -> &str {
        &self.schema
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        let _ = self
            .client
            .batch_execute(&format!("drop schema if exists {} cascade", self.schema));
    }
}

pub(crate) fn migrate(base_url: &str) -> Result<TestDatabase, String> {
    let schema = format!("lkjmc_test_{}", Uuid::new_v4().simple());
    let mut control =
        lkjmc_store::pool::connect_single(base_url).map_err(|error| error.to_string())?;
    control
        .batch_execute(&format!("create schema {schema}"))
        .map_err(|error| error.to_string())?;
    let database_url = lkjmc_store::pool::with_search_path(base_url, &schema)
        .map_err(|error| error.to_string())?;
    let database_schema = schema.clone();
    let result = (|| {
        let mut client =
            lkjmc_store::pool::connect_single(&database_url).map_err(|error| error.to_string())?;
        client
            .batch_execute("set lock_timeout = 0; set statement_timeout = 0")
            .map_err(|error| error.to_string())?;
        lkjmc_store::migrate::apply(&mut client).map_err(|error| error.to_string())?;
        Ok(TestDatabase {
            client,
            database_url,
            schema: database_schema,
        })
    })();
    if result.is_err() {
        let _ = control.batch_execute(&format!("drop schema if exists {schema} cascade"));
    }
    result
}
