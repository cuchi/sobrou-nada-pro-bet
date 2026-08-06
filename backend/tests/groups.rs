mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn create_group(router: &axum::Router, jwt: &str, name: &str) -> Value {
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
    serde_json::from_slice(&body_bytes).unwrap()
}

async fn login(router: &axum::Router) -> String {
    let (_, body) = common::post_json(router, "/api/dev/login", json!({})).await;
    body["token"].as_str().unwrap().to_string()
}

async fn get(router: &axum::Router, jwt: &str, uri: String) -> (StatusCode, Value) {
    let req = Request::get(uri)
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap_or(json!({}));
    (status, body)
}

async fn post(router: &axum::Router, jwt: &str, uri: &str) -> (StatusCode, Value) {
    let req = Request::post(uri)
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap_or(json!({}));
    (status, body)
}

#[tokio::test]
async fn create_and_view_group() {
    let (router, _pool) = common::app().await;
    let jwt = login(&router).await;
    let group = create_group(&router, &jwt, "Test Group").await;
    let group_id = group["id"].as_str().unwrap();

    let (status, body) = get(&router, &jwt, format!("/api/groups/{group_id}")).await;
    assert!(status.is_success());
    assert_eq!(body["group"]["name"], "Test Group");
    assert_eq!(body["members"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn list_my_groups() {
    let (router, _pool) = common::app().await;
    let jwt = login(&router).await;
    create_group(&router, &jwt, "Alpha").await;
    create_group(&router, &jwt, "Beta").await;

    let (status, body) = get(&router, &jwt, "/api/groups".into()).await;
    assert!(status.is_success());
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn cannot_view_group_as_non_member() {
    let (router, _pool) = common::app().await;
    let owner_jwt = login(&router).await;
    let group = create_group(&router, &owner_jwt, "Private").await;
    let group_id = group["id"].as_str().unwrap();

    let outsider_jwt = login(&router).await;
    let (status, _) = get(&router, &outsider_jwt, format!("/api/groups/{group_id}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn invite_code_owner_only() {
    let (router, _pool) = common::app().await;
    let owner_jwt = login(&router).await;
    let group = create_group(&router, &owner_jwt, "Invite").await;
    let group_id = group["id"].as_str().unwrap();

    // Owner can view
    let (status, body) = get(
        &router,
        &owner_jwt,
        format!("/api/groups/{group_id}/invite"),
    )
    .await;
    assert!(status.is_success());
    assert!(body["invite_code"].is_string());

    // Non-owner (member) cannot view
    // join as second user first to prove it's owner-scoped, not member-scoped
    let member_jwt = login(&router).await;
    let code = body["invite_code"].as_str().unwrap();
    let (jstatus, _) = post(&router, &member_jwt, &format!("/api/groups/join/{code}")).await;
    assert!(jstatus.is_success());

    let (status, _) = get(
        &router,
        &member_jwt,
        format!("/api/groups/{group_id}/invite"),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn invite_code_group_not_found() {
    let (router, _pool) = common::app().await;
    let jwt = login(&router).await;
    let (status, _) = get(
        &router,
        &jwt,
        "/api/groups/00000000-0000-0000-0000-000000000000/invite".into(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn regenerate_invite_owner_only() {
    let (router, _pool) = common::app().await;
    let owner_jwt = login(&router).await;
    let group = create_group(&router, &owner_jwt, "Rotate").await;
    let group_id = group["id"].as_str().unwrap();
    let old = group["invite_code"].as_str().unwrap().to_string();

    // Non-owner cannot regenerate
    let outsider_jwt = login(&router).await;
    let (status, _) = post(
        &router,
        &outsider_jwt,
        &format!("/api/groups/{group_id}/invite"),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Owner regenerates — code rotates
    let (status, body) = post(
        &router,
        &owner_jwt,
        &format!("/api/groups/{group_id}/invite"),
    )
    .await;
    assert!(status.is_success());
    let new = body["invite_code"].as_str().unwrap();
    assert_ne!(new, old);
}

#[tokio::test]
async fn join_group_errors() {
    let (router, _pool) = common::app().await;
    let owner_jwt = login(&router).await;
    let group = create_group(&router, &owner_jwt, "Join").await;

    // Invalid code → 404
    let joiner_jwt = login(&router).await;
    let (status, _) = post(&router, &joiner_jwt, "/api/groups/join/boguscode").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Valid join → success
    let code = group["invite_code"].as_str().unwrap();
    let (status, body) = post(&router, &joiner_jwt, &format!("/api/groups/join/{code}")).await;
    assert!(status.is_success());
    assert_eq!(body["group"]["id"], group["id"]);

    // Already a member → 400
    let (status, _) = post(&router, &joiner_jwt, &format!("/api/groups/join/{code}")).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn leaderboard_works() {
    let (router, _pool) = common::app().await;
    let owner_jwt = login(&router).await;
    let group = create_group(&router, &owner_jwt, "LB").await;
    let group_id = group["id"].as_str().unwrap();
    let code = group["invite_code"].as_str().unwrap();

    // Add a second member
    let second_jwt = login(&router).await;
    let (status, _) = post(&router, &second_jwt, &format!("/api/groups/join/{code}")).await;
    assert!(status.is_success());

    let (status, body) = get(
        &router,
        &owner_jwt,
        format!("/api/groups/{group_id}/leaderboard"),
    )
    .await;
    assert!(status.is_success());
    let entries = body.as_array().unwrap();
    assert_eq!(entries.len(), 2);
    assert!(entries[0]["balance"].is_number());
    assert!(entries[0]["betted"].is_number());
}

#[tokio::test]
async fn leaderboard_non_member_forbidden() {
    let (router, _pool) = common::app().await;
    let owner_jwt = login(&router).await;
    let group = create_group(&router, &owner_jwt, "LB2").await;
    let group_id = group["id"].as_str().unwrap();

    let outsider_jwt = login(&router).await;
    let (status, _) = get(
        &router,
        &outsider_jwt,
        format!("/api/groups/{group_id}/leaderboard"),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn leaderboard_sums_pending_bets() {
    let (router, pool) = common::app().await;
    let jwt = login(&router).await;
    let group = create_group(&router, &jwt, "LB3").await;
    let group_id: String = group["id"].as_str().unwrap().to_string();
    let group_uuid = uuid::Uuid::parse_str(&group_id).unwrap();

    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM users LIMIT 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    let event_id = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO events (id, external_id, home_team, away_team, championship, start_time, status, home_odds, draw_odds, away_odds)
         VALUES ($1, 'lb-evt', 'A', 'B', 'C', NOW() + INTERVAL '2 hours', 'scheduled', 1.5, 3.0, 4.0)",
    )
    .bind(event_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO bets (id, user_id, group_id, event_id, prediction, amount, odds, status)
         VALUES (gen_random_uuid(), $1, $2, $3, 'home_win', 200, 1.5, 'pending')",
    )
    .bind(user_id)
    .bind(group_uuid)
    .bind(event_id)
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = get(&router, &jwt, format!("/api/groups/{group_id}/leaderboard")).await;
    assert!(status.is_success());
    let entry = &body.as_array().unwrap()[0];
    // balance = 1000 (default) + 200 pending = 1200, betted = 200
    assert_eq!(entry["betted"].as_f64().unwrap(), 200.0);
    assert_eq!(entry["balance"].as_f64().unwrap(), 1200.0);
}
