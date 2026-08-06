mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

async fn seed_event(pool: &PgPool, status: &str, start_offset: &str) -> String {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO events (id, external_id, home_team, away_team, championship, start_time, status, home_odds, draw_odds, away_odds)
         VALUES ($1, $2, 'Flamengo', 'Vasco', 'Brasileirão', NOW() + $3::interval, $4, 1.5, 3.0, 4.0)",
    )
    .bind(id)
    .bind(format!("evt-{id}"))
    .bind(start_offset)
    .bind(status)
    .execute(pool)
    .await
    .unwrap();
    id.to_string()
}

async fn login(router: &axum::Router) -> String {
    let (_, body) = common::post_json(router, "/api/dev/login", json!({})).await;
    body["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn lists_events_without_filter() {
    let (router, pool) = common::app().await;
    seed_event(&pool, "scheduled", "+2 hours").await;
    seed_event(&pool, "live", "+1 hours").await;

    let jwt = login(&router).await;
    let req = Request::get("/api/events")
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert!(resp.status().is_success());
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let events: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(events.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn lists_events_with_status_filter() {
    let (router, pool) = common::app().await;
    seed_event(&pool, "scheduled", "+2 hours").await;
    seed_event(&pool, "live", "+1 hours").await;

    let jwt = login(&router).await;
    let req = Request::get("/api/events?status=scheduled")
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let events: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(events.as_array().unwrap().len(), 1);
    assert_eq!(events[0]["status"], "scheduled");
}

#[tokio::test]
async fn lists_events_with_comma_separated_statuses() {
    let (router, pool) = common::app().await;
    seed_event(&pool, "scheduled", "+2 hours").await;
    seed_event(&pool, "live", "+1 hours").await;
    seed_event(&pool, "finished", "-3 hours").await;

    let jwt = login(&router).await;
    let req = Request::get("/api/events?status=scheduled,live")
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let events: Value = serde_json::from_slice(&body_bytes).unwrap();
    let statuses: Vec<_> = events
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["status"].as_str().unwrap())
        .collect();
    // Ordered by start_time ascending: live (+1h) before scheduled (+2h)
    assert_eq!(statuses, vec!["live", "scheduled"]);
}

#[tokio::test]
async fn events_require_auth() {
    let (router, _pool) = common::app().await;
    let (status, _) = common::get_json(&router, "/api/events").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
