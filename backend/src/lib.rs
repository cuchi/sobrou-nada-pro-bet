pub mod auth;
pub mod db;
pub mod error;
pub mod models;
pub mod routes;

pub use main::build_app;

mod main {
    use axum::{Router, routing::get};
    use tower_http::{
        cors::{Any, CorsLayer},
        services::{ServeDir, ServeFile},
        trace::TraceLayer,
    };

    pub async fn build_app(pool: sqlx::PgPool) -> Router {
        let cors = build_cors();

        Router::new()
            .route("/health", get(crate::routes::health_check))
            .route(
                "/api/auth/google",
                axum::routing::post(crate::routes::google_login),
            )
            .route("/api/auth/me", get(crate::routes::me))
            .route(
                "/api/dev/login",
                axum::routing::post(crate::routes::dev_login),
            )
            .route(
                "/api/groups",
                get(crate::routes::list_my_groups).post(crate::routes::create_group),
            )
            .route("/api/groups/{id}", get(crate::routes::get_group))
            .route(
                "/api/groups/{id}/invite",
                get(crate::routes::get_invite).post(crate::routes::regenerate_invite),
            )
            .route(
                "/api/groups/join/{code}",
                axum::routing::post(crate::routes::join_group),
            )
            .route(
                "/api/groups/{id}/leaderboard",
                get(crate::routes::leaderboard),
            )
            .route("/api/events", get(crate::routes::list_events))
            .route(
                "/admin/events/sync",
                axum::routing::post(crate::routes::admin::sync_events),
            )
            .route(
                "/admin/bets/resolve",
                axum::routing::post(crate::routes::admin::resolve_bets),
            )
            .route(
                "/api/bets",
                get(crate::routes::list_bets).post(crate::routes::create_bet),
            )
            .route(
                "/api/bets/{id}/resolve",
                axum::routing::patch(crate::routes::resolve_bet),
            )
            .fallback_service(
                ServeDir::new("dist").not_found_service(ServeFile::new("dist/index.html")),
            )
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(
                        tower_http::trace::DefaultMakeSpan::new().level(tracing::Level::INFO),
                    )
                    .on_request(
                        tower_http::trace::DefaultOnRequest::new().level(tracing::Level::INFO),
                    )
                    .on_response(
                        tower_http::trace::DefaultOnResponse::new()
                            .level(tracing::Level::INFO)
                            .latency_unit(tower_http::LatencyUnit::Millis),
                    ),
            )
            .layer(cors)
            .with_state(pool)
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
}
