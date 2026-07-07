#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]
#![allow(missing_docs)]
use axum::body::Body;
use http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_200_with_json() {
    let app = brust_web::create_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}
