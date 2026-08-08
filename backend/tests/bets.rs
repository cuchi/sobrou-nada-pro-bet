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

/// Event that kicks off in 10 minutes — inside the 1h betting cutoff.
async fn seed_event_soon(pool: &PgPool) -> String {
    let id = Uuid::new_v4();
    let external_id = format!("test-{id}");
    sqlx::query(
        "INSERT INTO events (id, external_id, home_team, away_team, championship, start_time, status, home_odds, draw_odds, away_odds)
         VALUES ($1, $2, 'Flamengo', 'Vasco', 'Brasileirão', NOW() + INTERVAL '10 minutes', 'scheduled', 1.5, 3.0, 4.0)",
    )
    .bind(id)
    .bind(&external_id)
    .execute(pool)
    .await
    .unwrap();
    id.to_string()
}

async fn create_group(router: &axum::Router, jwt: &str, name: &str) -> String {
    let req = Request::post("/api/groups")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::from(
            serde_json::to_string(&json!({"name": name})).unwrap(),
        ))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let group: Value = serde_json::from_slice(&body_bytes).unwrap();
    group["id"].as_str().unwrap().to_string()
}

fn bet_request(jwt: &str, group_id: &str, event_id: &str, amount: u32) -> Request<Body> {
    Request::post("/api/bets")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::from(
            serde_json::to_string(&json!({
                "group_id": group_id,
                "event_id": event_id,
                "prediction": "home_win",
                "amount": amount,
                "odds": 1.5
            }))
            .unwrap(),
        ))
        .unwrap()
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

#[tokio::test]
async fn cannot_list_bets_outside_group() {
    let (router, _pool) = common::app().await;

    let (_, login) = common::post_json(&router, "/api/dev/login", json!({})).await;
    let owner_jwt = login["token"].as_str().unwrap().to_string();
    let group_id = create_group(&router, &owner_jwt, "Private Group").await;

    // Second user — not a member of the group
    let (_, login2) = common::post_json(&router, "/api/dev/login", json!({})).await;
    let outsider_jwt = login2["token"].as_str().unwrap().to_string();

    let req = Request::get(format!("/api/bets?group_id={group_id}"))
        .header("Authorization", format!("Bearer {outsider_jwt}"))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn cannot_bet_after_cutoff() {
    let (router, pool) = common::app().await;

    let (_, login) = common::post_json(&router, "/api/dev/login", json!({})).await;
    let jwt = login["token"].as_str().unwrap().to_string();
    let group_id = create_group(&router, &jwt, "Late Group").await;

    // Event starts in 10 minutes — inside the 1h cutoff
    let event_id = seed_event_soon(&pool).await;

    let req = bet_request(&jwt, &group_id, &event_id, 50);
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn cannot_bet_outside_group() {
    let (router, _pool) = common::app().await;

    let (_, login) = common::post_json(&router, "/api/dev/login", json!({})).await;
    let owner_jwt = login["token"].as_str().unwrap().to_string();
    let group_id = create_group(&router, &owner_jwt, "Members Only").await;

    let (_, login2) = common::post_json(&router, "/api/dev/login", json!({})).await;
    let outsider_jwt = login2["token"].as_str().unwrap().to_string();

    let req = bet_request(
        &outsider_jwt,
        &group_id,
        "00000000-0000-0000-0000-000000000000",
        50,
    );
    let resp = router.clone().oneshot(req).await.unwrap();
    // Phase D contract: `not_group_member` maps to 403 Forbidden.
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ── Dev-only single-bet resolver ─────────────────────

/// Place a pending bet as `jwt` and return its id as a `String`.
async fn place_pending_bet(
    router: &axum::Router,
    jwt: &str,
    group_id: &str,
    event_id: &str,
) -> String {
    let req = bet_request(jwt, group_id, event_id, 100);
    let resp = router.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(status.is_success(), "place_pending_bet failed: {body}");
    body["id"].as_str().unwrap().to_string()
}

fn dev_resolve_request(jwt: &str, bet_id: &str, outcome: &str) -> Request<Body> {
    Request::post("/api/dev/resolve-bet")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::from(
            serde_json::to_string(&json!({
                "bet_id": bet_id,
                "outcome": outcome
            }))
            .unwrap(),
        ))
        .unwrap()
}

#[tokio::test]
async fn dev_resolve_bet_marks_winner_and_credits_balance() {
    let (router, pool) = common::app().await;

    let (_, login) = common::post_json(&router, "/api/dev/login", json!({})).await;
    let jwt = login["token"].as_str().unwrap();

    let group_id = create_group(&router, jwt, "Dev Resolve Group").await;
    let event_id = seed_event(&pool).await;
    let bet_id = place_pending_bet(&router, jwt, &group_id, &event_id).await;

    // Resolve as home_win — matches the prediction the helper placed.
    let req = dev_resolve_request(jwt, &bet_id, "home_win");
    let resp = router.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(
        status.is_success(),
        "dev_resolve_bet failed: {status} {body}"
    );
    assert_eq!(body["outcome"], "home_win");
    assert_eq!(body["resolved"], 1);

    // Bet status should now be 'won'.
    let bet_status: String =
        sqlx::query_scalar("SELECT status::text FROM bets WHERE id = $1::uuid")
            .bind(&bet_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(bet_status, "won");

    // Event scores should be 1-0 (home_win synthetic).
    let scores: (i32, i32) =
        sqlx::query_as("SELECT home_score, away_score FROM events WHERE id = $1::uuid")
            .bind(&event_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(scores, (1, 0));

    // Balance went up by amount * odds (100 * 1.5 = 150) over the
    // original 1000 starting balance minus the 100 already deducted.
    let balance_after: f64 = sqlx::query_scalar(
        "SELECT gm.balance FROM group_members gm WHERE gm.group_id = $1::uuid LIMIT 1",
    )
    .bind(&group_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    // 1000 start − 100 bet + 150 payout = 1050
    assert!(
        (balance_after - 1050.0).abs() < 0.001,
        "balance_after={balance_after}, expected=1050"
    );
}

#[tokio::test]
async fn dev_resolve_bet_rejects_already_resolved_bet() {
    let (router, pool) = common::app().await;

    let (_, login) = common::post_json(&router, "/api/dev/login", json!({})).await;
    let jwt = login["token"].as_str().unwrap();

    let group_id = create_group(&router, jwt, "Double Resolve Group").await;
    let event_id = seed_event(&pool).await;
    let bet_id = place_pending_bet(&router, jwt, &group_id, &event_id).await;

    // First resolve succeeds.
    let req = dev_resolve_request(jwt, &bet_id, "home_win");
    let resp = router.clone().oneshot(req).await.unwrap();
    assert!(resp.status().is_success());

    // Second resolve: bet is now 'won', not 'pending'.
    let req = dev_resolve_request(jwt, &bet_id, "draw");
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn dev_resolve_bet_rejects_invalid_outcome() {
    let (router, pool) = common::app().await;

    let (_, login) = common::post_json(&router, "/api/dev/login", json!({})).await;
    let jwt = login["token"].as_str().unwrap();

    let group_id = create_group(&router, jwt, "Bad Outcome Group").await;
    let event_id = seed_event(&pool).await;
    let bet_id = place_pending_bet(&router, jwt, &group_id, &event_id).await;

    let req = dev_resolve_request(jwt, &bet_id, "pizza_party");
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
