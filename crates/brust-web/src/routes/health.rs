//! GET /health — liveness probe.
use axum::{Json, response::IntoResponse};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

/// Liveness probe — returns `{"status":"ok"}` with HTTP 200.
pub async fn handler() -> impl IntoResponse {
    Json(HealthResponse { status: "ok" })
}
