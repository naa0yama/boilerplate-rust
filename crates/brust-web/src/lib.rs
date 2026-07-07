//! brust-web: minimal Axum + `DaisyUI` + `OTel` web server boilerplate.

pub mod assets;
pub mod cli;
pub mod routes;
pub mod telemetry;
pub mod trace;

/// Returns the crate version string.
#[must_use]
pub const fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Creates the application router with all routes registered.
pub fn create_router() -> axum::Router {
    axum::Router::new()
        .route("/", axum::routing::get(routes::index::handler))
        .route("/health", axum::routing::get(routes::health::handler))
}
