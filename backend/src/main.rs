mod auth;
mod db;
mod error;
mod models;
mod routes;

use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Load .env from the backend directory
    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sobrou_nada_pro_bet=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Required env vars
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (check .env file)");
    if std::env::var("JWT_SECRET").is_err() {
        tracing::warn!(
            "JWT_SECRET not set — auth will fail. Generate one with: openssl rand -base64 32"
        );
    }
    if std::env::var("GOOGLE_CLIENT_ID").is_err() {
        tracing::warn!("GOOGLE_CLIENT_ID not set — Google login will fail.");
    }

    let pool = db::init(&database_url).await;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // Health
        .route("/health", get(routes::health_check))
        // Auth
        .route(
            "/api/auth/google",
            axum::routing::post(routes::google_login),
        )
        .route("/api/auth/me", get(routes::me))
        // Bets
        .route("/api/bets", get(routes::list_bets).post(routes::create_bet))
        .route(
            "/api/bets/{id}/resolve",
            axum::routing::patch(routes::resolve_bet),
        )
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("🚀 Backend running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}
