mod common;

use axum::http::{Request, StatusCode};
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
    // Email notifications defaults: opted-in, English locale.
    assert_eq!(body["user"]["email_notifications"], true);
    assert_eq!(body["user"]["locale"], "en");
}

#[tokio::test]
async fn auth_required() {
    let (router, _pool) = common::app().await;
    let (status, _) = common::get_json(&router, "/api/auth/me").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn me_returns_user_with_locale_and_email_toggle() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    let (router, pool) = common::app().await;

    // Create a user via dev_login (gives us a JWT), then update the
    // user via SQL so we control the locale + email_notifications
    // values that /me should echo back.
    let (_, login) = common::post_json(&router, "/api/dev/login", json!({})).await;
    let jwt = login["token"].as_str().unwrap().to_string();

    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(login["user"]["email"].as_str().unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();

    sqlx::query("UPDATE users SET locale = 'pt-BR', email_notifications = FALSE WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let req = Request::get("/api/auth/me")
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(body["user"]["locale"], "pt-BR");
    assert_eq!(body["user"]["email_notifications"], false);
}

async fn patch_me(
    router: &axum::Router,
    jwt: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    use axum::body::Body;
    use tower::ServiceExt;
    let req = Request::patch("/api/auth/me")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {jwt}"))
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap_or(json!({}));
    (status, body)
}

#[tokio::test]
async fn patch_me_updates_email_notifications() {
    let (router, _pool) = common::app().await;
    let (_, login) = common::post_json(&router, "/api/dev/login", json!({})).await;
    let jwt = login["token"].as_str().unwrap().to_string();

    let (status, body) = patch_me(&router, &jwt, json!({ "email_notifications": false })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user"]["email_notifications"], false);
    assert_eq!(body["user"]["locale"], "en"); // unchanged

    // Flip back on.
    let (status, body) = patch_me(&router, &jwt, json!({ "email_notifications": true })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user"]["email_notifications"], true);
}

#[tokio::test]
async fn patch_me_updates_locale() {
    let (router, _pool) = common::app().await;
    let (_, login) = common::post_json(&router, "/api/dev/login", json!({})).await;
    let jwt = login["token"].as_str().unwrap().to_string();

    let (status, body) = patch_me(&router, &jwt, json!({ "locale": "pt-BR" })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user"]["locale"], "pt-BR");
    assert_eq!(body["user"]["email_notifications"], true); // unchanged
}

#[tokio::test]
async fn patch_me_updates_both_at_once() {
    let (router, _pool) = common::app().await;
    let (_, login) = common::post_json(&router, "/api/dev/login", json!({})).await;
    let jwt = login["token"].as_str().unwrap().to_string();

    let (status, body) = patch_me(
        &router,
        &jwt,
        json!({ "email_notifications": false, "locale": "pt-BR" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user"]["email_notifications"], false);
    assert_eq!(body["user"]["locale"], "pt-BR");
}

#[tokio::test]
async fn patch_me_rejects_empty_body() {
    let (router, _pool) = common::app().await;
    let (_, login) = common::post_json(&router, "/api/dev/login", json!({})).await;
    let jwt = login["token"].as_str().unwrap().to_string();

    let (status, _) = patch_me(&router, &jwt, json!({})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn patch_me_rejects_oversized_locale() {
    let (router, _pool) = common::app().await;
    let (_, login) = common::post_json(&router, "/api/dev/login", json!({})).await;
    let jwt = login["token"].as_str().unwrap().to_string();

    let (status, _) = patch_me(&router, &jwt, json!({ "locale": "this-is-way-too-long" })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn patch_me_requires_auth() {
    let (router, _pool) = common::app().await;
    let (status, _) = patch_me(&router, "", json!({ "email_notifications": false })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
