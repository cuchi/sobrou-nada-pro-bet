#![allow(dead_code)]

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

pub async fn test_db() -> PgPool {
    let url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://snpb:snpb@localhost:5432/snpb_test".into());

    // Create the test DB if it doesn't exist (connect to default DB first)
    let admin_url = "postgres://snpb:snpb@localhost:5432/snpb";
    let admin_pool = PgPool::connect(admin_url).await.ok();
    if let Some(p) = &admin_pool {
        sqlx::query("CREATE DATABASE snpb_test")
            .execute(p)
            .await
            .ok();
        p.close().await;
    }

    let pool = PgPool::connect(&url)
        .await
        .expect("Failed to connect to test DB");

    // Serialize access across test binaries via advisory lock
    sqlx::query("SELECT pg_advisory_lock(42)")
        .execute(&pool)
        .await
        .expect("Failed to acquire lock");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    sqlx::query(
        "DO $$ DECLARE r RECORD; BEGIN
            FOR r IN (SELECT tablename FROM pg_tables WHERE schemaname = 'public' AND tablename != '_sqlx_migrations') LOOP
                EXECUTE 'TRUNCATE TABLE ' || quote_ident(r.tablename) || ' CASCADE';
            END LOOP;
        END $$",
    )
    .execute(&pool)
    .await
    .expect("Failed to truncate tables");

    // Lock released when connection drops at end of test
    pool
}

pub async fn app() -> (axum::Router, PgPool) {
    dotenvy::dotenv().ok();
    if std::env::var("JWT_SECRET").is_err() {
        unsafe {
            std::env::set_var("JWT_SECRET", "test-secret");
        }
    }
    if std::env::var("ADMIN_TOKEN").is_err() {
        unsafe {
            std::env::set_var("ADMIN_TOKEN", "test-admin-token");
        }
    }
    let pool = test_db().await;
    let router = sobrou_nada_pro_bet::build_app(pool.clone()).await;
    (router, pool)
}

pub async fn get_json(router: &axum::Router, uri: &str) -> (StatusCode, Value) {
    let req = Request::get(uri).body(Body::empty()).unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap_or(json!({}));
    (status, body)
}

pub async fn post_json(router: &axum::Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let req = Request::post(uri)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap_or(json!({}));
    (status, body)
}
