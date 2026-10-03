pub mod auth;
pub mod commands;
pub mod config;
pub mod deployment;
pub mod economy;
pub mod error;
pub mod hosting;
#[path = "../../hosting_limits.rs"]
pub mod hosting_limits;
pub mod pages;
pub mod presets;
pub mod queries;
pub mod server_tools;
pub mod services;
pub mod social;
pub mod timeline;
pub mod world;

use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct App {
    pub presets: Arc<Vec<presets::Preset>>,
    pub db: PgPool,
    pub config: Arc<config::Config>,
    pub http: reqwest::Client,
}

impl App {
    pub async fn connect(config: config::Config) -> anyhow::Result<Self> {
        config.validate()?;
        let db = sqlx::postgres::PgPoolOptions::new()
            .max_connections(24)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .connect(&config.database_url)
            .await?;
        sqlx::migrate!("../../migrations").run(&db).await?;
        tokio::fs::create_dir_all(config.storage.join("artifacts")).await?;
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(15))
            .user_agent("lkjmc/0.1 (https://lkjmc.lkjsxc.com)")
            .build()?;
        Ok(Self {
            presets: Arc::new(presets::load(config.server_presets.as_deref())?),
            db,
            config: Arc::new(config),
            http,
        })
    }
}

pub fn router(app: App) -> axum::Router {
    use axum::{
        Router,
        extract::DefaultBodyLimit,
        http::{HeaderValue, header},
        routing::{get, post},
    };
    use tower_http::{
        services::{ServeDir, ServeFile},
        set_header::SetResponseHeaderLayer,
        trace::TraceLayer,
    };
    Router::new()
        .route(
            "/health/live",
            get(|| async { axum::Json(serde_json::json!({"live":true})) }),
        )
        .route("/health/ready", get(queries::ready))
        .route(
            "/internal/v1/jobs/{id}/backup",
            post(services::backup_control),
        )
        .route(
            "/internal/v1/jobs/{id}/backup-prune",
            post(services::backup_prune),
        )
        .route(
            "/internal/v1/official-backups/{id}/database",
            get(services::backup_download),
        )
        .route("/auth/login", get(auth::login))
        .route("/auth/callback", get(auth::callback))
        .route("/auth/logout", post(auth::logout))
        .route("/api/v1/me", get(queries::me))
        .route("/api/v1/server-presets", get(presets::list))
        .route("/api/v1/home", get(pages::home))
        .route("/api/v1/timeline", get(timeline::read))
        .route("/api/v1/rooms", get(timeline::room_list))
        .route("/api/v1/history/{kind}", get(pages::history))
        .route("/api/v1/servers/{id}", get(pages::server))
        .route("/api/v1/view/{view}", get(queries::page_view))
        .route("/api/v1/rooms/{id}/messages", get(queries::messages))
        .route("/api/v1/players", get(queries::players))
        .route("/api/v1/jobs/{id}", get(server_tools::read_job))
        .route("/api/v1/reports/{id}", get(queries::report))
        .route("/api/v1/reports/preview", post(queries::report_preview))
        .route("/api/v1/commands", post(commands::http_command))
        .route("/api/v1/voice/{room}", post(services::voice_token))
        .route(
            "/api/v1/servers/{id}/artifacts",
            post(hosting::upload).layer(DefaultBodyLimit::max(1024 * 1024 * 1024)),
        )
        .route("/internal/v1/poll", post(server_tools::poll))
        .route(
            "/internal/v1/jobs/{id}/ack",
            post(server_tools::ack).layer(DefaultBodyLimit::max(16 * 1024 * 1024)),
        )
        .route("/internal/v1/observations", post(services::observe))
        .route("/internal/v1/worlds/ready", post(services::world_ready))
        .route(
            "/internal/v1/jobs/{id}/identity-ready",
            post(services::identity_ready),
        )
        .route(
            "/internal/v1/jobs/{id}/identity-state",
            get(services::identity_status),
        )
        .route("/internal/v1/game/connect", post(services::game_connect))
        .route(
            "/internal/v1/jobs/{id}/context",
            post(services::host_context),
        )
        .route("/internal/v1/game/route", post(services::game_route))
        .route(
            "/internal/v1/game/linked/{native_id}",
            get(services::game_linked),
        )
        .route(
            "/internal/v1/game/heartbeat",
            post(services::game_heartbeat),
        )
        .route(
            "/internal/v1/game/disconnect",
            post(services::game_disconnect),
        )
        .route("/internal/v1/game/command", post(services::game_command))
        .route("/internal/v1/game/event", post(services::game_event))
        .route(
            "/internal/v1/game/profile/{native_id}",
            get(services::game_profile),
        )
        .route("/internal/v1/game/view", post(services::game_view))
        .route("/internal/v1/spawn/reserve", post(world::reserve_spawn))
        .route("/internal/v1/spawn/resolve", post(world::resolve_spawn))
        .route("/internal/v1/projection", get(services::projection))
        .route("/internal/v1/artifacts/{id}", get(services::artifact))
        .route(
            "/api/{*path}",
            get(|| async { error::Error::missing() }).post(|| async { error::Error::missing() }),
        )
        .route(
            "/internal/{*path}",
            get(|| async { error::Error::missing() }).post(|| async { error::Error::missing() }),
        )
        .fallback_service(
            ServeDir::new(&app.config.web)
                .not_found_service(ServeFile::new(app.config.web.join("index.html"))),
        )
        .layer(DefaultBodyLimit::max(256 * 1024))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("same-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(app)
}
