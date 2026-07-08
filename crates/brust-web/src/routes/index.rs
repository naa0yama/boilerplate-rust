//! GET / — index page.

use askama::Template;
use axum::response::{Html, IntoResponse};

#[derive(Template)]
#[template(path = "pages/index.html")]
struct IndexTemplate;

/// Renders the index page as HTML.
pub async fn handler() -> impl IntoResponse {
    IndexTemplate.render().map_or_else(
        |_| axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(), // NOTEST(unreachable): Askama render() never fails for a compile-validated template
        |html| Html(html).into_response(),
    )
}
