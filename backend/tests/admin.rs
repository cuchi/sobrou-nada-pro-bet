mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn sync_requires_valid_token() {
    let (router, _pool) = common::app().await;

    let req = Request::post("/admin/events/sync")
        .header("X-Admin-Token", "wrong")
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn resolve_requires_valid_token() {
    let (router, _pool) = common::app().await;

    let req = Request::post("/admin/bets/resolve")
        .header("X-Admin-Token", "wrong")
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
