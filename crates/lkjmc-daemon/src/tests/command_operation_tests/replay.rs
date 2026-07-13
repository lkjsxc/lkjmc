use super::*;

#[test]
fn duplicate_mutations_pass() -> Result<(), String> {
    let Some(url) = database_url() else {
        return Ok(());
    };
    let mut database = crate::test_database::migrate(&url)?;
    let state = state(database.url().to_string());
    let id = uuid::Uuid::new_v4().to_string();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    let (first, second) = runtime.block_on(async {
        let first = dispatch_admitted(&state, request(&id, "ja")?).await?;
        let second = dispatch_admitted(&state, request(&id, "ja")?).await?;
        Ok::<_, String>((first, second))
    })?;
    assert!(first.ok);
    assert_eq!(first, second, "identical replay changed its response");
    let outcome = lkjmc_store::command::lookup(database.client_mut(), &id)
        .map_err(|error| error.to_string())?
        .ok_or("journal outcome missing")?;
    assert_eq!(outcome.result, "succeeded");
    assert_eq!(outcome.response, first);
    assert_eq!(settings(database.client_mut())?, (1, "ja".into()));
    Ok(())
}

#[test]
fn conflicting_duplicate_is_denied() -> Result<(), String> {
    let Some(url) = database_url() else {
        return Ok(());
    };
    let mut database = crate::test_database::migrate(&url)?;
    let state = state(database.url().to_string());
    let id = uuid::Uuid::new_v4().to_string();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    let (body_conflict, actor_conflict) = runtime.block_on(async {
        let first = dispatch_admitted(&state, request(&id, "en")?).await?;
        assert!(first.ok);
        let body = dispatch_admitted(&state, request(&id, "ja")?).await?;
        let mut changed_actor = request(&id, "en")?;
        changed_actor.actor.name = "different-actor".into();
        let actor = dispatch_admitted(&state, changed_actor).await?;
        Ok::<_, String>((body, actor))
    })?;
    for conflict in [&body_conflict, &actor_conflict] {
        assert!(!conflict.ok);
        assert_eq!(error_code(conflict), "request.id_conflict");
    }
    let mut changed_command = request(&id, "en")?;
    changed_command.command = "player.settings.hud".into();
    let command_conflict =
        lkjmc_store::command::execute_desired(database.client_mut(), &changed_command, |_| {
            Err(lkjmc_store::error::StoreError::invalid_state(
                "conflicting command invoked its mutation",
            ))
        })
        .map_err(|error| error.to_string())?;
    assert!(matches!(
        command_conflict,
        lkjmc_store::command::Execution::Conflict
    ));
    let outcome = lkjmc_store::command::lookup(database.client_mut(), &id)
        .map_err(|error| error.to_string())?
        .ok_or("journal outcome missing")?;
    assert_eq!(outcome.result, "succeeded");
    assert_eq!(
        outcome
            .response
            .body
            .as_ref()
            .and_then(|body| body["language"].as_str()),
        Some("en")
    );
    assert_eq!(settings(database.client_mut())?, (1, "en".into()));
    Ok(())
}

fn settings(client: &mut postgres::Client) -> Result<(i64, String), String> {
    let player = uuid::Uuid::parse_str(PLAYER).map_err(|error| error.to_string())?;
    let row = client
        .query_one(
            "select count(*)::bigint, max(language) from player_settings where player_uuid = $1",
            &[&player],
        )
        .map_err(|error| error.to_string())?;
    Ok((row.get(0), row.get(1)))
}
