mod auth;
mod db;
mod error;
mod models;
mod routes;

use std::panic;

use axum::{routing::get, Router};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    panic::set_hook(Box::new(|info| {
        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| info.payload().downcast_ref::<String>().map(|s| s.as_str()))
            .unwrap_or("unknown panic");
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".into());
        tracing::error!(%payload, %location, "Panic caught");
    }));

    let _ = dotenvy::dotenv();

    let default_filter: String = match std::env::var("RUST_LOG") {
        Ok(_) => "".into(),
        Err(_) => {
            let is_prod = std::env::var("ENVIRONMENT")
                .map(|v| v == "production")
                .unwrap_or(false);
            if is_prod {
                "sobrou_nada_pro_bet=info,tower_http=info".into()
            } else {
                "sobrou_nada_pro_bet=debug,tower_http=debug".into()
            }
        }
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| default_filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (check .env file)");

    if std::env::var("JWT_SECRET").is_err() {
        panic!("JWT_SECRET must be set. Generate one with: openssl rand -base64 32");
    }
    if std::env::var("GOOGLE_CLIENT_ID").is_err() {
        panic!("GOOGLE_CLIENT_ID must be set in .env");
    }

    let pool = db::init(&database_url).await;
    let cors = build_cors();

    let app = Router::new()
        .route("/health", get(routes::health_check))
        .route(
            "/api/auth/google",
            axum::routing::post(routes::google_login),
        )
        .route("/api/auth/me", get(routes::me))
        .route("/api/dev/login", axum::routing::post(routes::dev_login))
        .route(
            "/api/groups",
            get(routes::list_my_groups).post(routes::create_group),
        )
        .route("/api/groups/{id}", get(routes::get_group))
        .route(
            "/api/groups/{id}/invite",
            get(routes::get_invite).post(routes::regenerate_invite),
        )
        .route(
            "/api/groups/join/{code}",
            axum::routing::post(routes::join_group),
        )
        .route("/api/groups/{id}/leaderboard", get(routes::leaderboard))
        .route("/api/bets", get(routes::list_bets).post(routes::create_bet))
        .route(
            "/api/bets/{id}/resolve",
            axum::routing::patch(routes::resolve_bet),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    tower_http::trace::DefaultMakeSpan::new().level(tracing::Level::INFO),
                )
                .on_request(tower_http::trace::DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(
                    tower_http::trace::DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                        .latency_unit(tower_http::LatencyUnit::Millis),
                ),
        )
        .layer(cors)
        .with_state(pool);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".into());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Listening on http://{addr}");

    axum::serve(listener, app).await.unwrap();
}

fn build_cors() -> CorsLayer {
    match std::env::var("CORS_ALLOWED_ORIGINS") {
        Ok(origins) if !origins.is_empty() => {
            let list: Vec<_> = origins
                .split(',')
                .map(|s| s.trim().parse().unwrap())
                .collect();
            CorsLayer::new()
                .allow_origin(list)
                .allow_methods(Any)
                .allow_headers(Any)
        }
        _ => {
            if std::env::var("ENVIRONMENT")
                .map(|v| v == "production")
                .unwrap_or(false)
            {
                tracing::warn!("CORS_ALLOWED_ORIGINS not set — all origins allowed");
            }
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        }
    }
}
