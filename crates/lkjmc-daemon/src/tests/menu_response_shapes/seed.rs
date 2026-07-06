use postgres::Client;
use serde_json::json;
use uuid::Uuid;

const INSTANCE_ID: &str = "shape-survival";

pub fn minimal_rows(client: &mut Client, player: Uuid, other: Uuid) -> Result<(), String> {
    identity(client, player, "ShapePlayer")?;
    identity(client, other, "ShapeGuide")?;
    settings(client, player)?;
    instance(client)?;
    home(client, player)?;
    warp(client)?;
    points(client, player)?;
    shop(client)?;
    achievement(client, player)?;
    kit(client)?;
    vote(client)?;
    mail(client, player, other)?;
    report(client, player, other)?;
    daily(client, player)?;
    party(client, player)?;
    claim(client, player)?;
    Ok(())
}

fn identity(client: &mut Client, player: Uuid, name: &str) -> Result<(), String> {
    store(lkjmc_store::player::insert_identity(client, player, name))
}

fn settings(client: &mut Client, player: Uuid) -> Result<(), String> {
    store(lkjmc_store::player_settings::set_language(
        client, player, "ja",
    ))?;
    store(lkjmc_store::player_settings::set_hud(client, player, true))?;
    store(lkjmc_store::player_settings::set_menu_enabled(
        client, player, false,
    ))
}

fn instance(client: &mut Client) -> Result<(), String> {
    store(lkjmc_store::instance::insert(
        client,
        INSTANCE_ID,
        None,
        "paper",
        "running",
        &json!({"serverPort": 25565, "connectHost": "127.0.0.1"}),
    ))?;
    store(lkjmc_store::instance::upsert_observation(
        client,
        INSTANCE_ID,
        "process-healthy",
        Some(4242),
        true,
        None,
    ))?;
    store(lkjmc_store::instance_presence::upsert_heartbeat(
        client,
        lkjmc_store::instance_presence::PresenceHeartbeat {
            instance_id: INSTANCE_ID,
            player_count: Some(3),
            max_players: Some(20),
            ready: true,
            implementation: Some("paper"),
        },
    ))?;
    store(lkjmc_store::proxy_registration::report(
        client,
        &[lkjmc_store::proxy_registration::RegistrationReport {
            instance_id: INSTANCE_ID,
            connect_host: "127.0.0.1",
            connect_port: 25565,
            registered: true,
            failure_reason: None,
        }],
    ))
}

fn home(client: &mut Client, player: Uuid) -> Result<(), String> {
    store(lkjmc_store::homes::upsert(
        client,
        Uuid::new_v4(),
        player,
        "base",
        INSTANCE_ID,
        json!({"world": "world", "x": 1.5, "y": 64.0, "z": -2.25}),
    ))
}

fn warp(client: &mut Client) -> Result<(), String> {
    store(lkjmc_store::warps::upsert(
        client,
        "spawn",
        INSTANCE_ID,
        json!({"world": "world", "x": 0.0, "y": 65.0, "z": 0.0}),
    ))
}

fn points(client: &mut Client, player: Uuid) -> Result<(), String> {
    store(lkjmc_store::points::grant(client, player, 50, "shape-test"))
}

fn shop(client: &mut Client) -> Result<(), String> {
    store(lkjmc_store::shop::upsert_item_with_metadata(
        client,
        "shape-diamond",
        "shop.item.shape-diamond",
        25,
        json!({
            "category": "blocks",
            "delivery": {"executor": "minecraft-item", "material": "DIAMOND", "amount": 3}
        }),
    ))
}

fn achievement(client: &mut Client, player: Uuid) -> Result<(), String> {
    store(lkjmc_store::achievement::grant(
        client,
        player,
        "shape-achievement",
        "achievement.shape.title",
    ))
}

fn kit(client: &mut Client) -> Result<(), String> {
    store(lkjmc_store::kits::upsert(
        client,
        "starter",
        "kit.starter.title",
        5,
        24,
    ))
}

fn vote(client: &mut Client) -> Result<(), String> {
    store(lkjmc_store::votes::upsert(
        client,
        "vote-site",
        "vote.site.title",
        "https://vote.example.invalid",
        10,
    ))
}

fn mail(client: &mut Client, player: Uuid, other: Uuid) -> Result<(), String> {
    store(lkjmc_store::mail::send(
        client,
        Uuid::new_v4(),
        player,
        other,
        "ShapeGuide",
        "Welcome to the shape test.",
    ))
}

fn report(client: &mut Client, player: Uuid, other: Uuid) -> Result<(), String> {
    store(lkjmc_store::reports::create(
        client,
        Uuid::new_v4(),
        player,
        other,
        INSTANCE_ID,
        "shape assertion",
    ))
}

fn daily(client: &mut Client, player: Uuid) -> Result<(), String> {
    store(lkjmc_store::daily::claim(client, player, 100)).map(|_| ())
}

fn party(client: &mut Client, player: Uuid) -> Result<(), String> {
    store(lkjmc_store::party::create(
        client,
        Uuid::new_v4(),
        player,
        "Shape Party",
    ))
}

fn claim(client: &mut Client, player: Uuid) -> Result<(), String> {
    store(lkjmc_store::claims::create_claim(
        client,
        lkjmc_store::claims::NewClaim {
            id: Uuid::new_v4(),
            owner_uuid: player,
            owner_name: "ShapePlayer",
            name: "spawn",
            instance_id: INSTANCE_ID,
            world_name: "world",
            chunk_x: 0,
            chunk_z: 0,
        },
    ))
    .map(|_| ())
}

fn store<T>(result: Result<T, lkjmc_store::error::StoreError>) -> Result<T, String> {
    result.map_err(|error| error.to_string())
}
