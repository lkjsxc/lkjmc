use serde_json::json;

use crate::error::CliError;
use crate::format;

pub fn migrate(database_url: &str, json_output: bool) -> Result<(), CliError> {
    let mut client = lkjmc_store::pool::connect(database_url)?;
    let applied = lkjmc_store::migrate::apply(&mut client)?;
    if json_output {
        format::print_json(&json!({"applied": applied}))
    } else {
        println!("ok db migrate {}", applied.len());
        Ok(())
    }
}

pub fn status(database_url: &str, json_output: bool) -> Result<(), CliError> {
    let mut client = lkjmc_store::pool::connect(database_url)?;
    let versions = lkjmc_store::migrate::applied_versions(&mut client)?;
    if json_output {
        format::print_json(&json!({"versions": versions}))
    } else {
        println!("ok db status {}", versions.len());
        Ok(())
    }
}

pub fn reset_test(database_url: &str) -> Result<(), CliError> {
    if std::env::var("LKJMC_TEST_RESET_DATABASE").ok().as_deref() != Some("1") {
        return Err(CliError::message("LKJMC_TEST_RESET_DATABASE=1 is required"));
    }
    let mut client = lkjmc_store::pool::connect(database_url)?;
    client
        .batch_execute("drop schema public cascade; create schema public")
        .map_err(lkjmc_store::error::StoreError::from)?;
    println!("ok db reset-test");
    Ok(())
}
