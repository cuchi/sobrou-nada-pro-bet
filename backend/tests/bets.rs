mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

async fn seed_event(pool: &PgPool) -> String {
    let id = Uuid::new_v4();
    let external_id = format!("test-{id}");
    sqlx::query(
        "INSERT INTO events (id, external_id, home_team, away_team, championship, start_time, status, home_odds, draw_odds, away_odds)
         VALUES ($1, $2, 'Flamengo', 'Vasco', 'Brasileirão', NOW() + INTERVAL '2 hours', 'scheduled', 1.5, 3.0, 4.0)",
    )
    .bind(id)
    .bind(&external_id)
    .execute(pool)
    .await
    .unwrap();
    id.to_string()
}

#[tokio::test]
async fn place_and_list_bet() {
    let (router, pool) = common::app().await;

    let (_, login) = common::post_json(&router, "/api/dev/login", json!({})).await;
    let jwt = login["token"].as_str().unwrap();

    // Create a group and join it (dev login already does this via /me groups)
    let req = Request::post("/api/groups")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::from(
            serde_json::to_string(&json!({"name": "Bet Test Group"})).unwrap(),
        ))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let group: Value = serde_json::from_slice(&body_bytes).unwrap();
    let group_id = group["id"].as_str().unwrap();

    // Seed an event
    let event_id = seed_event(&pool).await;

    // Place a bet
    common::post_json(
        &router,
        "/api/bets",
        json!({
            "group_id": group_id,
            "event_id": event_id,
            "prediction": "home_win",
            "amount": 100,
            "odds": 1.5
        }),
    )
    .await;
    // Need auth — redo with header
    let req = Request::post("/api/bets")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::from(
            serde_json::to_string(&json!({
                "group_id": group_id,
                "event_id": event_id,
                "prediction": "home_win",
                "amount": 100,
                "odds": 1.5
            }))
            .unwrap(),
        ))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(status.is_success(), "Bet creation failed: {body}");

    // List bets by group
    let req = Request::get(format!("/api/bets?group_id={group_id}"))
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert!(resp.status().is_success());
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let bets: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(bets.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn cannot_bet_twice_same_event() {
    let (router, pool) = common::app().await;

    let (_, login) = common::post_json(&router, "/api/dev/login", json!({})).await;
    let jwt = login["token"].as_str().unwrap();

    let req = Request::post("/api/groups")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::from(
            serde_json::to_string(&json!({"name": "Dup Test"})).unwrap(),
        ))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let group: Value = serde_json::from_slice(&body_bytes).unwrap();
    let group_id = group["id"].as_str().unwrap();

    let event_id = seed_event(&pool).await;

    let bet_body = json!({
        "group_id": group_id,
        "event_id": event_id,
        "prediction": "home_win",
        "amount": 50,
        "odds": 1.5
    });

    // First bet succeeds
    let req = Request::post("/api/bets")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::from(serde_json::to_string(&bet_body).unwrap()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert!(resp.status().is_success());

    // Second bet fails
    let req = Request::post("/api/bets")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::from(serde_json::to_string(&bet_body).unwrap()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn cannot_bet_with_insufficient_balance() {
    let (router, pool) = common::app().await;

    let (_, login) = common::post_json(&router, "/api/dev/login", json!({})).await;
    let jwt = login["token"].as_str().unwrap();

    let req = Request::post("/api/groups")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::from(
            serde_json::to_string(&json!({"name": "Poor Test"})).unwrap(),
        ))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let group: Value = serde_json::from_slice(&body_bytes).unwrap();
    let group_id = group["id"].as_str().unwrap();

    let event_id = seed_event(&pool).await;

    let req = Request::post("/api/bets")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::from(
            serde_json::to_string(&json!({
                "group_id": group_id,
                "event_id": event_id,
                "prediction": "home_win",
                "amount": 9999,
                "odds": 1.5
            }))
            .unwrap(),
        ))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
