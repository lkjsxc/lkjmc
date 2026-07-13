use super::*;

#[test]
fn journal_failure_rolls_back_mutation() -> Result<(), String> {
    let Some(url) = database_url() else {
        return Ok(());
    };
    let mut database = crate::test_database::migrate(&url)?;
    database
        .client_mut()
        .batch_execute(
            "create function reject_journal_success() returns trigger language plpgsql as $$
             begin if new.result = 'succeeded' then
             raise exception 'injected journal failure'; end if; return new; end $$;
             create trigger reject_journal_success before update on commands
             for each row execute function reject_journal_success();",
        )
        .map_err(|error| error.to_string())?;
    let state = state(database.url().to_string());
    let id = uuid::Uuid::new_v4().to_string();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    let response = runtime.block_on(dispatch_admitted(&state, request(&id, "ja")?))?;
    assert!(!response.ok);
    assert_eq!(error_code(&response), "database.error");
    let outcome = lkjmc_store::command::lookup(database.client_mut(), &id)
        .map_err(|error| error.to_string())?
        .ok_or("journal outcome missing")?;
    assert_eq!(outcome.result, "failed");
    let player = uuid::Uuid::parse_str(PLAYER).map_err(|error| error.to_string())?;
    let count: i64 = database
        .client_mut()
        .query_one(
            "select count(*) from player_settings where player_uuid = $1",
            &[&player],
        )
        .map_err(|error| error.to_string())?
        .get(0);
    assert_eq!(count, 0, "mutation committed without journal success");
    Ok(())
}

#[test]
fn failed_worker_leaves_no_requested_journal() -> Result<(), String> {
    let Some(url) = database_url() else {
        return Ok(());
    };
    let mut database = crate::test_database::migrate(&url)?;
    database
        .client_mut()
        .batch_execute(
            "create function reject_settings() returns trigger language plpgsql as $$
             begin raise exception 'injected mutation failure'; end $$;
             create trigger reject_settings before insert or update on player_settings
             for each row execute function reject_settings();",
        )
        .map_err(|error| error.to_string())?;
    let state = state(database.url().to_string());
    let id = uuid::Uuid::new_v4().to_string();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    let response = runtime.block_on(dispatch_admitted(&state, request(&id, "ja")?))?;
    assert!(!response.ok);
    assert_eq!(error_code(&response), "database.error");
    let outcome = lkjmc_store::command::lookup(database.client_mut(), &id)
        .map_err(|error| error.to_string())?
        .ok_or("journal outcome missing")?;
    assert_eq!(outcome.result, "failed");
    assert_eq!(outcome.response, response);
    let running: i64 = database
        .client_mut()
        .query_one(
            "select count(*) from commands where result = 'requested'",
            &[],
        )
        .map_err(|error| error.to_string())?
        .get(0);
    assert_eq!(running, 0, "finished worker left a requested journal row");
    Ok(())
}
