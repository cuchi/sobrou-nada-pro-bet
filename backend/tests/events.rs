mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

/// Seed an event with a stored DB status and a specific start time offset.
async fn seed_event(pool: &PgPool, stored_status: &str, start_offset: &str) -> String {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO events (id, external_id, home_team, away_team, championship, start_time, status, home_odds, draw_odds, away_odds)
         VALUES ($1, $2, 'Flamengo', 'Vasco', 'Brasileirão', NOW() + $3::interval, $4, 1.5, 3.0, 4.0)",
    )
    .bind(id)
    .bind(format!("evt-{id}"))
    .bind(start_offset)
    .bind(stored_status)
    .execute(pool)
    .await
    .unwrap();
    id.to_string()
}

async fn login(router: &axum::Router) -> String {
    let (_, body) = common::post_json(router, "/api/dev/login", json!({})).await;
    body["token"].as_str().unwrap().to_string()
}

async fn list(router: &axum::Router, jwt: &str, uri: &str) -> Value {
    let req = Request::get(uri)
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert!(resp.status().is_success());
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&body_bytes).unwrap()
}

#[tokio::test]
async fn derives_scheduled_live_finished_from_start_time() {
    let (router, pool) = common::app().await;
    // Stored as 'scheduled' (that's all sync ever stores)
    seed_event(&pool, "scheduled", "+2 hours").await; // future → scheduled
    seed_event(&pool, "scheduled", "-30 minutes").await; // in window → live
    seed_event(&pool, "scheduled", "-3 hours").await; // past window → finished (waiting)

    let jwt = login(&router).await;
    let events = list(&router, &jwt, "/api/events").await;

    let statuses: Vec<_> = events
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["status"].as_str().unwrap())
        .collect();
    // Ordered by start_time ascending: -3h, -30min, +2h
    assert_eq!(statuses, vec!["finished", "live", "scheduled"]);
}

#[tokio::test]
async fn preserves_stored_finished_and_cancelled() {
    let (router, pool) = common::app().await;
    seed_event(&pool, "finished", "-5 hours").await; // stored finished
    seed_event(&pool, "cancelled", "+1 hours").await; // stored cancelled (future but cancelled)

    let jwt = login(&router).await;
    let events = list(&router, &jwt, "/api/events").await;

    let statuses: Vec<_> = events
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["status"].as_str().unwrap())
        .collect();
    assert_eq!(statuses, vec!["finished", "cancelled"]);
}

#[tokio::test]
async fn filter_by_derived_status() {
    let (router, pool) = common::app().await;
    seed_event(&pool, "scheduled", "+2 hours").await; // scheduled
    seed_event(&pool, "scheduled", "-30 minutes").await; // live
    seed_event(&pool, "scheduled", "-3 hours").await; // finished (waiting)

    let jwt = login(&router).await;
    let events = list(&router, &jwt, "/api/events?status=live").await;
    let statuses: Vec<_> = events
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["status"].as_str().unwrap())
        .collect();
    assert_eq!(statuses, vec!["live"]);
}

#[tokio::test]
async fn filter_by_comma_separated_statuses() {
    let (router, pool) = common::app().await;
    seed_event(&pool, "scheduled", "-30 minutes").await; // live
    seed_event(&pool, "scheduled", "+2 hours").await; // scheduled

    let jwt = login(&router).await;
    let events = list(&router, &jwt, "/api/events?status=scheduled,live").await;
    assert_eq!(events.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn events_require_auth() {
    let (router, _pool) = common::app().await;
    let (status, _) = common::get_json(&router, "/api/events").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
