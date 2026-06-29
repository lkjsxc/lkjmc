#[cfg(test)]
mod tests {
    use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
    use lkjmc_core::id::CommandId;
    use serde_json::{json, Value};

    use crate::app::AppState;

    const PLAYER: &str = "00000000-0000-0000-0000-000000000701";

    #[test]
    fn menu_data_commands_return_documented_shapes_when_database_configured() -> Result<(), String>
    {
        let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
            return Ok(());
        };
        let mut guard = reset_and_migrate(&database_url)?;
        let state = state(database_url);
        for (command, body, shape) in cases() {
            let response = call(&state, command, body)?;
            assert_shape(&response, shape)?;
        }
        guard
            .batch_execute("select pg_advisory_unlock(752647)")
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    enum Shape {
        Array(&'static str),
        Bool(&'static str),
        Daily,
        Balance,
    }

    fn cases() -> Vec<(&'static str, Value, Shape)> {
        vec![
            ("instance.list", json!({}), Shape::Array("instances")),
            (
                "player.home.list",
                json!({"playerUuid": PLAYER}),
                Shape::Array("homes"),
            ),
            ("player.warp.list", json!({}), Shape::Array("warps")),
            (
                "claim.list",
                json!({"ownerUuid": PLAYER}),
                Shape::Array("claims"),
            ),
            ("player.shop.list", json!({}), Shape::Array("items")),
            ("player.kit.list", json!({}), Shape::Array("kits")),
            ("player.vote.list", json!({}), Shape::Array("links")),
            (
                "player.mail.inbox",
                json!({"playerUuid": PLAYER, "limit": 14}),
                Shape::Array("messages"),
            ),
            (
                "player.report.list",
                json!({"limit": 14}),
                Shape::Array("reports"),
            ),
            (
                "player.daily.status",
                json!({"playerUuid": PLAYER}),
                Shape::Daily,
            ),
            (
                "player.points.balance",
                json!({"playerUuid": PLAYER, "name": "Alex"}),
                Shape::Balance,
            ),
            (
                "player.achievements.list",
                json!({"playerUuid": PLAYER}),
                Shape::Array("achievements"),
            ),
            (
                "player.party.info",
                json!({"playerUuid": PLAYER}),
                Shape::Bool("found"),
            ),
        ]
    }

    fn assert_shape(value: &Value, shape: Shape) -> Result<(), String> {
        match shape {
            Shape::Array(key) => value
                .get(key)
                .and_then(Value::as_array)
                .map(|_| ())
                .ok_or_else(|| format!("missing array key: {key}")),
            Shape::Bool(key) => value
                .get(key)
                .and_then(Value::as_bool)
                .map(|_| ())
                .ok_or_else(|| format!("missing bool key: {key}")),
            Shape::Daily => {
                assert_shape(value, Shape::Bool("claimedToday"))?;
                value
                    .get("points")
                    .and_then(Value::as_i64)
                    .map(|_| ())
                    .ok_or_else(|| "missing points".to_string())
            }
            Shape::Balance => value
                .get("balance")
                .and_then(Value::as_i64)
                .map(|_| ())
                .ok_or_else(|| "missing balance".to_string()),
        }
    }

    fn reset_and_migrate(database_url: &str) -> Result<postgres::Client, String> {
        let mut client =
            lkjmc_store::pool::connect(database_url).map_err(|error| error.to_string())?;
        client
            .batch_execute(
                "select pg_advisory_lock(752647); drop schema public cascade; create schema public",
            )
            .map_err(|error| error.to_string())?;
        lkjmc_store::migrate::apply(&mut client).map_err(|error| error.to_string())?;
        Ok(client)
    }

    fn state(database_url: String) -> AppState {
        AppState::with_config_path(
            Some(database_url),
            "/tmp/lkjmc-config".to_string(),
            "/tmp/lkjmc-logs".to_string(),
            "/tmp/lkjmc-jars".to_string(),
            "/tmp/lkjmc-data".to_string(),
            None,
        )
    }

    fn call(state: &AppState, command: &str, body: Value) -> Result<Value, String> {
        let response = crate::api::dispatch(state, request(command, body)?);
        if response.ok {
            return response
                .body
                .ok_or_else(|| "missing response body".to_string());
        }
        Err(response
            .error
            .map(|error| format!("{}: {}", error.code, error.message))
            .unwrap_or_else(|| "unknown error".to_string()))
    }

    fn request(command: &str, body: Value) -> Result<CommandEnvelope, String> {
        Ok(CommandEnvelope {
            request_id: CommandId::parse("request id", command)
                .map_err(|error| error.to_string())?,
            actor: Actor {
                kind: ActorKind::Cli,
                name: "menu-shape-test".to_string(),
            },
            command: command.to_string(),
            body,
        })
    }
}
