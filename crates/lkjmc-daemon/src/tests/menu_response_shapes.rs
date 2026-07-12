mod assertions;
mod seed;

use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;

const PLAYER: &str = "00000000-0000-0000-0000-000000000701";
const OTHER: &str = "00000000-0000-0000-0000-000000000702";
const INSTANCE_ID: &str = "shape-survival";
const HOME: &str = "base";

#[test]
fn menu_data_commands_return_documented_shapes_when_database_configured() -> Result<(), String> {
    let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let mut guard = crate::test_database::migrate(&database_url)?;
    seed::minimal_rows(guard.client_mut(), uuid(PLAYER)?, uuid(OTHER)?)?;
    let state = state(guard.url().to_string());
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    runtime.block_on(async {
        for case in cases() {
            let request = request(case.command, case.body)?;
            let response = if case.command == "player.settings.get" {
                dispatch_admitted(&state, request).await?
            } else {
                crate::dispatch::dispatch(&state, request)
            };
            if case.command == "player.settings.get" {
                let body = response.body.ok_or("settings response body missing")?;
                (case.assertion)(&body).map_err(|error| format!("{}: {error}", case.command))?;
            } else {
                assert!(!response.ok, "{} unexpectedly ran", case.command);
                assert_eq!(
                    response.error.map(|error| error.code),
                    Some("command.effect_denied".to_string()),
                    "{}",
                    case.command
                );
            }
        }
        Ok(())
    })
}

struct Case {
    command: &'static str,
    body: Value,
    assertion: fn(&Value) -> Result<(), String>,
}

impl Case {
    fn new(
        command: &'static str,
        body: Value,
        assertion: fn(&Value) -> Result<(), String>,
    ) -> Self {
        Self {
            command,
            body,
            assertion,
        }
    }
}

fn cases() -> Vec<Case> {
    vec![
        Case::new("instance.list", json!({}), assertions::instance_list),
        Case::new(
            "player.home.list",
            json!({"playerUuid": PLAYER}),
            assertions::home_list,
        ),
        Case::new(
            "player.home.get",
            json!({"playerUuid": PLAYER, "home": HOME}),
            assertions::home_get,
        ),
        Case::new("player.warp.list", json!({}), assertions::warp_list),
        Case::new("player.shop.list", json!({}), assertions::shop_list),
        Case::new(
            "player.achievements.list",
            json!({"playerUuid": PLAYER}),
            assertions::achievement_list,
        ),
        Case::new(
            "player.random-teleport.quote",
            json!({"playerUuid": PLAYER, "serverId": INSTANCE_ID, "profileId": "nether"}),
            assertions::random_teleport_quote,
        ),
        Case::new(
            "player.settings.get",
            json!({"playerUuid": PLAYER}),
            assertions::settings_get,
        ),
        Case::new(
            "player.points.balance",
            json!({"playerUuid": PLAYER, "name": "ShapePlayer"}),
            assertions::points_balance,
        ),
        Case::new("player.kit.list", json!({}), assertions::kit_list),
        Case::new("player.vote.list", json!({}), assertions::vote_list),
        Case::new(
            "player.mail.inbox",
            json!({"playerUuid": PLAYER, "limit": 14}),
            assertions::mail_inbox,
        ),
        Case::new(
            "player.report.list",
            json!({"limit": 14}),
            assertions::report_list,
        ),
        Case::new(
            "player.daily.status",
            json!({"playerUuid": PLAYER}),
            assertions::daily_status,
        ),
        Case::new(
            "player.party.info",
            json!({"playerUuid": PLAYER}),
            assertions::party_info,
        ),
        Case::new(
            "adventure.catalog.list",
            json!({}),
            assertions::adventure_catalog,
        ),
        Case::new(
            "claim.list",
            json!({"ownerUuid": PLAYER}),
            assertions::claim_list,
        ),
    ]
}

async fn dispatch_admitted(
    state: &AppState,
    request: CommandEnvelope,
) -> Result<lkjmc_core::command::CommandResponse, String> {
    let admission = state
        .admit_request()
        .ok_or("request admission unavailable")?;
    let state = state.clone();
    admission
        .run_blocking(move || crate::dispatch::dispatch(&state, request))
        .await
        .map_err(|_| "request worker did not complete".to_string())
}

fn state(database_url: String) -> AppState {
    AppState::with_config_path(
        Some(database_url),
        8,
        "/tmp/lkjmc-config".to_string(),
        "/tmp/lkjmc-logs".to_string(),
        "/tmp/lkjmc-jars".to_string(),
        "/tmp/lkjmc-data".to_string(),
        None,
        None,
        None,
    )
}

fn request(command: &str, body: Value) -> Result<CommandEnvelope, String> {
    Ok(CommandEnvelope {
        request_id: CommandId::parse("request id", command).map_err(|error| error.to_string())?,
        actor: Actor {
            kind: ActorKind::Cli,
            name: "menu-shape-test".to_string(),
        },
        command: command.to_string(),
        body,
    })
}

fn uuid(value: &str) -> Result<Uuid, String> {
    Uuid::parse_str(value).map_err(|error| error.to_string())
}
