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
#[allow(clippy::panic)]
fn panicked_mutation_releases_transaction_lock() -> Result<(), String> {
    let Some(url) = database_url() else {
        return Ok(());
    };
    let database = crate::test_database::migrate(&url)?;
    let pool = lkjmc_store::pool::build(database.url(), 2, std::time::Duration::from_secs(2))
        .map_err(|error| error.to_string())?;
    let mut retry_connection = pool.get().map_err(|error| error.to_string())?;
    let retry_pid: i32 = retry_connection
        .query_one("select pg_backend_pid()", &[])
        .map_err(|error| error.to_string())?
        .get(0);
    let id = uuid::Uuid::new_v4().to_string();
    let panic_request = request(&id, "ja")?;
    let panic_pool = pool.clone();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    let state = state(database.url().to_string());
    let admission = state
        .admit_request()
        .ok_or("request admission unavailable")?;
    let joined = runtime.block_on(admission.run_blocking(move || {
        let mut connection = match panic_pool.get() {
            Ok(connection) => connection,
            Err(error) => {
                return Err(lkjmc_store::error::StoreError::invalid_state(
                    error.to_string(),
                ));
            }
        };
        lkjmc_store::command::execute_desired(&mut connection, &panic_request, |_| {
            panic!("injected mutation panic after journal insertion")
        })
    }));
    assert!(matches!(joined, Err(crate::app::BlockingError::Join)));

    let mut reused = pool.get().map_err(|error| error.to_string())?;
    let reused_pid: i32 = reused
        .query_one("select pg_backend_pid()", &[])
        .map_err(|error| error.to_string())?
        .get(0);
    assert_ne!(reused_pid, retry_pid, "panic connection was not reused");
    let requested: i64 = reused
        .query_one(
            "select count(*) from commands where id = $1 and result = 'requested'",
            &[&id],
        )
        .map_err(|error| error.to_string())?
        .get(0);
    assert_eq!(requested, 0, "panic left a requested journal row");
    drop(reused);

    let retry_request = request(&id, "ja")?;
    let player = uuid::Uuid::parse_str(PLAYER).map_err(|error| error.to_string())?;
    let retry = lkjmc_store::command::execute_desired(
        &mut retry_connection,
        &retry_request,
        |transaction| {
            lkjmc_store::player_settings::set_language_for_identity(
                transaction,
                player,
                "Repeat",
                "ja",
            )?;
            Ok(serde_json::json!({"playerUuid": PLAYER, "language": "ja"}))
        },
    )
    .map_err(|error| error.to_string())?;
    let lkjmc_store::command::Execution::Outcome(response) = retry else {
        return Err("identical retry conflicted".into());
    };
    assert!(response.ok, "panic retry did not produce success");
    let replay =
        lkjmc_store::command::execute_desired(&mut retry_connection, &retry_request, |_| {
            Err(lkjmc_store::error::StoreError::invalid_state(
                "stable replay invoked mutation",
            ))
        })
        .map_err(|error| error.to_string())?;
    let lkjmc_store::command::Execution::Outcome(replayed) = replay else {
        return Err("stable replay conflicted".into());
    };
    assert_eq!(replayed, response);
    let outcome = lkjmc_store::command::lookup(&mut retry_connection, &id)
        .map_err(|error| error.to_string())?
        .ok_or("panic retry journal outcome missing")?;
    assert_eq!(outcome.result, "succeeded");
    let settings: (i64, String) = retry_connection
        .query_one(
            "select count(*)::bigint, max(language) from player_settings where player_uuid = $1",
            &[&player],
        )
        .map_err(|error| error.to_string())
        .map(|row| (row.get(0), row.get(1)))?;
    assert_eq!(settings, (1, "ja".into()));
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
