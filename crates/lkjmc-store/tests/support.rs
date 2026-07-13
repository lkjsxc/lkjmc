use lkjmc_store::{audit, jar, pool};
use uuid::Uuid;

pub const TEST_SHA: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

pub struct TestDatabase {
    client: postgres::Client,
    url: String,
    schema: String,
}

impl TestDatabase {
    pub fn client_mut(&mut self) -> &mut postgres::Client {
        &mut self.client
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        let _ = self
            .client
            .batch_execute(&format!("drop schema if exists {} cascade", self.schema));
    }
}

pub fn database() -> Result<Option<TestDatabase>, lkjmc_store::error::StoreError> {
    let Ok(base_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(None);
    };
    let schema = format!("lkjmc_test_{}", Uuid::new_v4().simple());
    let mut control = pool::connect(&base_url)?;
    control.batch_execute(&format!("create schema {schema}"))?;
    let setup = (|| {
        let url = pool::with_search_path(&base_url, &schema)?;
        let client = pool::connect(&url)?;
        Ok(TestDatabase {
            client,
            url,
            schema: schema.clone(),
        })
    })();
    if setup.is_err() {
        let _ = control.batch_execute(&format!("drop schema if exists {schema} cascade"));
    }
    setup.map(Some)
}

pub fn new_jar(id: Uuid) -> jar::NewJarAsset<'static> {
    jar::NewJarAsset {
        id,
        kind: "paper",
        project: "paper",
        channel: "stable",
        name: "paper-test.jar",
        path: "/opt/lkjmc/jars/papermc/paper/paper-test.jar",
        sha256: TEST_SHA,
        size_bytes: 3,
        source: "test",
    }
}

pub fn new_audit(id: Uuid) -> audit::NewAuditEvent<'static> {
    audit::NewAuditEvent {
        id,
        actor_kind: "cli",
        actor_name: "test",
        action: "instance.create",
        target_kind: "instance",
        target_id: "hub",
        result: "succeeded",
    }
}
