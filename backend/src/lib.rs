pub mod auth;
pub mod db;
pub mod email;
pub mod env;
pub mod error;
pub mod models;
pub mod routes;

pub use app::build_app;

mod app {
    use axum::{
        Router,
        routing::{get, post},
    };
    use tower_http::{
        cors::{Any, CorsLayer},
        services::{ServeDir, ServeFile},
        trace::TraceLayer,
    };

    use crate::routes::{
        admin, create_bet, create_group, dev_login, dev_resolve_bet, get_group, get_invite,
        google_login, health_check, join_group, leaderboard, list_bets, list_events,
        list_my_groups, me, patch_me, regenerate_invite,
    };

    pub async fn build_app(pool: sqlx::PgPool) -> Router {
        Router::new()
            .route("/health", get(health_check))
            .route("/api/auth/google", post(google_login))
            .route("/api/auth/me", get(me).patch(patch_me))
            .route("/api/dev/login", post(dev_login))
            .route("/api/dev/resolve-bet", post(dev_resolve_bet))
            .route("/api/groups", get(list_my_groups).post(create_group))
            .route("/api/groups/{id}", get(get_group))
            .route(
                "/api/groups/{id}/invite",
                get(get_invite).post(regenerate_invite),
            )
            .route("/api/groups/join/{code}", post(join_group))
            .route("/api/groups/{id}/leaderboard", get(leaderboard))
            .route("/api/events", get(list_events))
            .route("/api/bets", get(list_bets).post(create_bet))
            .route("/admin/events/sync", post(admin::sync_events))
            .route("/admin/bets/resolve", post(admin::resolve_bets))
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
            .layer(build_cors())
            .with_state(pool)
    }

    fn build_cors() -> CorsLayer {
        match &crate::env::ENV.cors_allowed_origins {
            Some(origins) => {
                let list: Vec<_> = origins
                    .split(',')
                    .map(|s| s.trim().parse().unwrap())
                    .collect();
                CorsLayer::new()
                    .allow_origin(list)
                    .allow_methods(Any)
                    .allow_headers(Any)
            }
            None => {
                if crate::env::ENV.is_prod() {
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
