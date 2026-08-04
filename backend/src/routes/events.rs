use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::models::Event;

// ── Events ────────────────────────────────────────────

#[derive(Deserialize)]
pub struct EventsQuery {
    status: Option<String>,
}

/// GET /api/events — list events, optionally filtered by status (comma-separated)
pub async fn list_events(
    _auth: AuthUser,
    State(pool): State<PgPool>,
    Query(query): Query<EventsQuery>,
) -> Result<Json<Value>, AppError> {
    let events: Vec<Event> = if let Some(status) = &query.status {
        // Split "scheduled,live" into ["scheduled", "live"]
        let statuses: Vec<&str> = status.split(',').map(|s| s.trim()).collect();
        // Use ANY() to match any of the given statuses
        sqlx::query_as("SELECT * FROM events WHERE status = ANY($1) ORDER BY start_time")
            .bind(&statuses)
            .fetch_all(&pool)
            .await
    } else {
        sqlx::query_as("SELECT * FROM events ORDER BY start_time")
            .fetch_all(&pool)
            .await
    }
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(json!(events)))
}
