use axum::Json;
use serde_json::{Value, json};

pub mod admin;
pub mod auth;
pub mod bets;
pub mod events;
pub mod groups;

// Re-exports so main.rs can use `routes::handler_name`
pub use auth::{dev_login, google_login, me};
pub use bets::{create_bet, list_bets};
pub use events::list_events;
pub use groups::{
    create_group, get_group, get_invite, join_group, leaderboard, list_my_groups, regenerate_invite,
};

/// GET /health
pub async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
