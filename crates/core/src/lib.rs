pub mod auth;
pub mod commands;
pub mod config;
pub mod economy;
pub mod error;
pub mod hosting;
pub mod queries;
pub mod services;
pub mod social;
pub mod world;

use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct App {
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
        .route("/auth/login", get(auth::login))
        .route("/auth/callback", get(auth::callback))
        .route("/auth/logout", post(auth::logout))
        .route("/api/v1/me", get(queries::me))
        .route("/api/v1/view/{view}", get(queries::view))
        .route("/api/v1/rooms/{id}/messages", get(queries::messages))
        .route("/api/v1/players", get(queries::players))
        .route("/api/v1/jobs/{id}", get(queries::job))
        .route("/api/v1/reports/{id}", get(queries::report))
        .route("/api/v1/reports/preview", post(queries::report_preview))
        .route("/api/v1/commands", post(commands::http_command))
        .route("/api/v1/voice/{room}", post(services::voice_token))
        .route(
            "/api/v1/servers/{id}/artifacts",
            post(hosting::upload).layer(DefaultBodyLimit::max(1024 * 1024 * 1024)),
        )
        .route("/internal/v1/poll", post(services::poll))
        .route(
            "/internal/v1/jobs/{id}/ack",
            post(services::ack).layer(DefaultBodyLimit::max(16 * 1024 * 1024)),
        )
        .route("/internal/v1/observations", post(services::observe))
        .route("/internal/v1/worlds/ready", post(services::world_ready))
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
