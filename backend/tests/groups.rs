mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;

#[tokio::test]
async fn create_and_view_group() {
    let (router, _pool) = common::app().await;

    let (_, login) = common::post_json(&router, "/api/dev/login", json!({})).await;
    let jwt = login["token"].as_str().unwrap();

    let req = Request::post("/api/groups")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::from(
            serde_json::to_string(&json!({"name": "Test Group"})).unwrap(),
        ))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let group: Value = serde_json::from_slice(&body_bytes).unwrap();
    let group_id = group["id"].as_str().unwrap();

    let req = Request::get(format!("/api/groups/{group_id}"))
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn invite_code_access() {
    let (router, _pool) = common::app().await;

    let (_, login) = common::post_json(&router, "/api/dev/login", json!({})).await;
    let jwt = login["token"].as_str().unwrap();

    let req = Request::post("/api/groups")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::from(
            serde_json::to_string(&json!({"name": "Invite Test"})).unwrap(),
        ))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let group: Value = serde_json::from_slice(&body_bytes).unwrap();
    let group_id = group["id"].as_str().unwrap();

    let req = Request::get(format!("/api/groups/{group_id}/invite"))
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert!(resp.status().is_success());
}
