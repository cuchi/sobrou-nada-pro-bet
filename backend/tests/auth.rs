mod common;

use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn health_check() {
    let (router, _pool) = common::app().await;
    let (status, body) = common::get_json(&router, "/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn dev_login_creates_user() {
    let (router, _pool) = common::app().await;
    let (status, body) = common::post_json(&router, "/api/dev/login", json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["token"].is_string());
    assert!(body["user"]["name"].is_string());
}

#[tokio::test]
async fn auth_required() {
    let (router, _pool) = common::app().await;
    let (status, _) = common::get_json(&router, "/api/auth/me").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
