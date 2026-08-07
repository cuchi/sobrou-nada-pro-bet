//! Integration tests for the Phase D error-code wire shape.
//!
//! These tests exercise the **in-scope** routes (`google_login`, `create_bet`,
//! `join_group`) and the `internal` 5xx shape, plus one **out-of-scope** route
//! that must keep its legacy `{ "error": "<string>" }` shape to prove we
//! didn't accidentally refactor it.
//!
//! Each test asserts the JSON body has the expected `code`, `params`, and
//! `message` fields, and that the HTTP status matches the contract's
//! variant → status mapping.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

// ── helpers ─────────────────────────────────────────────

async fn read_response(res: axum::response::Response) -> (StatusCode, Value) {
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap_or(json!({}));
    (status, value)
}

async fn post(
    router: &axum::Router,
    uri: &str,
    jwt: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut req = Request::post(uri).header("Content-Type", "application/json");
    if let Some(jwt) = jwt {
        req = req.header("Authorization", format!("Bearer {jwt}"));
    }
    let req = req
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    read_response(router.clone().oneshot(req).await.unwrap()).await
}

async fn get_no_auth(router: &axum::Router, uri: &str) -> (StatusCode, Value) {
    let req = Request::get(uri).body(Body::empty()).unwrap();
    read_response(router.clone().oneshot(req).await.unwrap()).await
}

async fn login(router: &axum::Router) -> String {
    let (_, body) = common::post_json(router, "/api/dev/login", json!({})).await;
    body["token"].as_str().unwrap().to_string()
}

async fn create_group(router: &axum::Router, jwt: &str, name: &str) -> Value {
    let req = Request::post("/api/groups")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::from(
            serde_json::to_string(&json!({"name": name})).unwrap(),
        ))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn seed_event(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO events (id, external_id, home_team, away_team, championship, start_time, status, home_odds, draw_odds, away_odds)
         VALUES ($1, $2, 'Flamengo', 'Vasco', 'Brasileirão', NOW() + INTERVAL '2 hours', 'scheduled', 1.5, 3.0, 4.0)",
    )
    .bind(id)
    .bind(format!("shape-evt-{id}"))
    .execute(pool)
    .await
    .unwrap();
    id
}

async fn seed_event_soon(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO events (id, external_id, home_team, away_team, championship, start_time, status, home_odds, draw_odds, away_odds)
         VALUES ($1, $2, 'Flamengo', 'Vasco', 'Brasileirão', NOW() + INTERVAL '10 minutes', 'scheduled', 1.5, 3.0, 4.0)",
    )
    .bind(id)
    .bind(format!("shape-evt-soon-{id}"))
    .execute(pool)
    .await
    .unwrap();
    id
}

// ── in-scope routes: structured shape ───────────────────

#[tokio::test]
async fn google_login_with_bad_credential_emits_structured_shape() {
    // The route reaches Google's tokeninfo endpoint, which will reject
    // our garbage credential. We don't care about the specific code here
    // (network failures vs parse failures are both possible in CI) — only
    // that the wire shape is the new one for each in-scope error.
    let (router, _pool) = common::app().await;
    let (status, body) = post(
        &router,
        "/api/auth/google",
        None,
        json!({"credential": "definitely-not-a-real-google-token"}),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    // Structured shape must be present.
    assert!(body.get("code").is_some(), "body was {body}");
    assert!(body.get("params").is_some(), "body was {body}");
    assert!(body.get("message").is_some(), "body was {body}");
    // Legacy shape must NOT be present.
    assert!(body.get("error").is_none(), "body was {body}");

    // The code must be one of the in-scope google_login codes.
    let code = body["code"].as_str().unwrap();
    assert!(
        code == "auth_google_failed" || code == "auth_google_invalid",
        "unexpected code {code:?}"
    );
    assert_eq!(body["params"], Value::Null);
}

#[tokio::test]
async fn create_bet_insufficient_balance_has_params() {
    let (router, pool) = common::app().await;
    let jwt = login(&router).await;
    let group = create_group(&router, &jwt, "Shape Insufficient").await;
    let group_id = group["id"].as_str().unwrap();
    let event_id = seed_event(&pool).await.to_string();

    let (status, body) = post(
        &router,
        "/api/bets",
        Some(&jwt),
        json!({
            "group_id": group_id,
            "event_id": event_id,
            "prediction": "home_win",
            "amount": 9999,
            "odds": 1.5,
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "insufficient_balance");
    assert_eq!(body["params"]["have"], 1000.0);
    assert_eq!(body["params"]["bet"], 9999.0);
    assert_eq!(
        body["message"],
        "Insufficient balance. You have 1000 points, bet is 9999."
    );
}

#[tokio::test]
async fn create_bet_not_group_member_is_403_with_null_params() {
    let (router, _pool) = common::app().await;

    // Owner creates a group; outsider tries to bet on it.
    let owner_jwt = login(&router).await;
    let group = create_group(&router, &owner_jwt, "Shape Outsider").await;
    let group_id = group["id"].as_str().unwrap();

    let outsider_jwt = login(&router).await;
    let (status, body) = post(
        &router,
        "/api/bets",
        Some(&outsider_jwt),
        json!({
            "group_id": group_id,
            "event_id": "00000000-0000-0000-0000-000000000000",
            "prediction": "home_win",
            "amount": 10,
            "odds": 1.5,
        }),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "not_group_member");
    assert_eq!(body["params"], Value::Null);
}

#[tokio::test]
async fn create_bet_already_bet_on_event_is_400() {
    let (router, pool) = common::app().await;
    let jwt = login(&router).await;
    let group = create_group(&router, &jwt, "Shape Dup").await;
    let group_id = group["id"].as_str().unwrap();
    let event_id = seed_event(&pool).await.to_string();

    let bet = json!({
        "group_id": group_id,
        "event_id": event_id,
        "prediction": "home_win",
        "amount": 50,
        "odds": 1.5,
    });

    // First bet succeeds.
    let (status, _) = post(&router, "/api/bets", Some(&jwt), bet.clone()).await;
    assert!(status.is_success(), "first bet status was {status}");

    // Second bet fails with the structured code.
    let (status, body) = post(&router, "/api/bets", Some(&jwt), bet).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "already_bet_on_event");
    assert_eq!(body["params"], Value::Null);
}

#[tokio::test]
async fn create_bet_event_not_found_is_400() {
    let (router, _pool) = common::app().await;
    let jwt = login(&router).await;
    let group = create_group(&router, &jwt, "Shape No Event").await;
    let group_id = group["id"].as_str().unwrap();

    let (status, body) = post(
        &router,
        "/api/bets",
        Some(&jwt),
        json!({
            "group_id": group_id,
            "event_id": "00000000-0000-0000-0000-000000000000",
            "prediction": "home_win",
            "amount": 10,
            "odds": 1.5,
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "event_not_found");
    assert_eq!(body["params"], Value::Null);
}

#[tokio::test]
async fn create_bet_betting_closed_is_400() {
    let (router, pool) = common::app().await;
    let jwt = login(&router).await;
    let group = create_group(&router, &jwt, "Shape Closed").await;
    let group_id = group["id"].as_str().unwrap();
    let event_id = seed_event_soon(&pool).await.to_string();

    let (status, body) = post(
        &router,
        "/api/bets",
        Some(&jwt),
        json!({
            "group_id": group_id,
            "event_id": event_id,
            "prediction": "home_win",
            "amount": 10,
            "odds": 1.5,
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "betting_closed");
    assert_eq!(body["params"], Value::Null);
}

#[tokio::test]
async fn join_group_invalid_code_is_404_structured() {
    let (router, _pool) = common::app().await;
    let jwt = login(&router).await;

    let req = Request::post("/api/groups/join/totally-bogus-code")
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let (status, body) = read_response(resp).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "invalid_invite_code");
    assert_eq!(body["params"], Value::Null);
    assert_eq!(body["message"], "Invalid invite code");
}

#[tokio::test]
async fn join_group_already_member_is_400_structured() {
    let (router, _pool) = common::app().await;
    let owner_jwt = login(&router).await;
    let group = create_group(&router, &owner_jwt, "Shape Already").await;
    let code = group["invite_code"].as_str().unwrap();

    // Second user joins successfully first.
    let joiner_jwt = login(&router).await;
    let req = Request::post(format!("/api/groups/join/{code}"))
        .header("Authorization", format!("Bearer {joiner_jwt}"))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert!(resp.status().is_success());

    // Then attempts to join again — second attempt returns the structured code.
    let req = Request::post(format!("/api/groups/join/{code}"))
        .header("Authorization", format!("Bearer {joiner_jwt}"))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let (status, body) = read_response(resp).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "already_in_group");
    assert_eq!(body["params"], Value::Null);
    assert_eq!(body["message"], "You're already in this group");
}

// ── canonical internal shape ────────────────────────────

#[tokio::test]
async fn internal_shape_via_unit_test() {
    // The AppError::Internal status mapping is verified directly via the
    // unit tests in `error.rs`. Here we just confirm the *unit test* path
    // exists by calling the same code path that routes hit.
    use serde_json::Value;
    use sobrou_nada_pro_bet::error::{AppError, ErrorCode};

    let fake = AppError::Internal("secret db detail".into());
    let resp = fake.into_response();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 64 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["code"], "internal");
    assert_eq!(body["params"], Value::Null);
    assert_eq!(body["message"], "Internal server error");
    // The internal detail must NOT leak into the body.
    assert!(!body.to_string().contains("secret db detail"));

    // Smoke check: InsufficientBalance's Display message matches the contract.
    let ic = ErrorCode::InsufficientBalance {
        have: 1.0,
        bet: 2.0,
    };
    assert_eq!(
        ic.to_string(),
        "Insufficient balance. You have 1 points, bet is 2."
    );
}

// ── out-of-scope routes: legacy shape must be preserved ──

#[tokio::test]
async fn auth_me_without_token_returns_legacy_shape() {
    // Hitting a route that uses the AuthUser extractor must still emit
    // the legacy `{ "error": "<string>" }` shape — that extractor is
    // out-of-scope for Phase D.
    let (router, _pool) = common::app().await;
    let (status, body) = get_no_auth(&router, "/api/auth/me").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], "Missing Authorization header");
    // No structured fields.
    assert!(body.get("code").is_none(), "body was {body}");
    assert!(body.get("params").is_none(), "body was {body}");
    assert!(body.get("message").is_none(), "body was {body}");
}

#[tokio::test]
async fn list_bets_outside_group_returns_legacy_shape() {
    let (router, _pool) = common::app().await;
    let owner_jwt = login(&router).await;
    let group = create_group(&router, &owner_jwt, "Shape Legacy").await;
    let group_id = group["id"].as_str().unwrap();

    let outsider_jwt = login(&router).await;
    let req = Request::get(format!("/api/bets?group_id={group_id}"))
        .header("Authorization", format!("Bearer {outsider_jwt}"))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let (status, body) = read_response(resp).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "You're not a member of this group");
    assert!(body.get("code").is_none(), "body was {body}");
}
