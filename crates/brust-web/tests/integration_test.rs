#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]
#![allow(missing_docs)]
use assert_cmd::cargo::cargo_bin_cmd;
use axum::body::Body;
use http::{Request, StatusCode};
use predicates::prelude::*;
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

#[tokio::test]
async fn index_returns_200_html() {
    let app = brust_web::create_router();
    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let headers = response.headers();
    assert!(
        headers["content-type"]
            .to_str()
            .unwrap()
            .contains("text/html")
    );
}

#[tokio::test]
async fn static_css_returns_200() {
    let app = brust_web::create_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/static/app.css")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn static_htmx_returns_200() {
    let app = brust_web::create_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/static/htmx.min.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[test]
#[cfg_attr(miri, ignore)]
fn version_subcommand_prints_version() {
    let mut cmd = cargo_bin_cmd!("brust-web");
    cmd.arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}
