use axum::{
    Json,
    extract::{Query, State},
};
use chrono::{Duration, Utc};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::models::Event;

// ── Events ────────────────────────────────────────────

#[derive(Deserialize)]
pub struct EventsQuery {
    status: Option<String>,
}

/// A reasonable upper bound for a match duration — used to tell apart a match
/// that is currently live from one that's already over but not yet resolved.
const MATCH_DURATION: Duration = Duration::hours(2);

/// Derive the status a user should see for an event.
///
/// The DB only persists `scheduled` (from sync) plus `finished` / `cancelled`
/// (from resolution). Transient `live` and "finished but waiting for results"
/// states are computed on the fly from the start time:
/// - `scheduled`   → starts in the future
/// - `live`        → started, still within the expected match window
/// - `finished`    → (stored) results are in
/// - `finished`    → (derived) the match window elapsed but results aren't synced yet
/// - `cancelled`   → (stored) the match was called off
fn display_status(event: &Event) -> &'static str {
    match event.status.as_str() {
        "finished" => "finished",
        "cancelled" => "cancelled",
        _ => {
            let now = Utc::now();
            if event.start_time > now {
                "scheduled"
            } else if now <= event.start_time + MATCH_DURATION {
                "live"
            } else {
                "finished"
            }
        }
    }
}

/// GET /api/events — list events, optionally filtered by status (comma-separated)
///
/// Filtering matches against the *displayed* (derived) status, so callers can
/// request `scheduled` or `live` even though those aren't stored literally.
pub async fn list_events(
    _auth: AuthUser,
    State(pool): State<PgPool>,
    Query(query): Query<EventsQuery>,
) -> Result<Json<Value>, AppError> {
    let events: Vec<Event> = sqlx::query_as("SELECT * FROM events ORDER BY start_time")
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let include: Option<Vec<String>> = query
        .status
        .map(|s| s.split(',').map(|x| x.trim().to_string()).collect());

    let visible: Vec<Value> = events
        .iter()
        .filter(|event| {
            include
                .as_ref()
                .is_none_or(|statuses| statuses.contains(&display_status(event).to_string()))
        })
        .map(|event| {
            let mut value = serde_json::to_value(event).unwrap_or(json!({}));
            value["status"] = json!(display_status(event));
            value
        })
        .collect();

    Ok(Json(json!(visible)))
}
