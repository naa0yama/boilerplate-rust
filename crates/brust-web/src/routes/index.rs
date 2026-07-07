//! GET / — index page.

use askama::Template;
use axum::response::{Html, IntoResponse};

#[derive(Template)]
#[template(path = "pages/index.html")]
struct IndexTemplate;

/// Renders the index page as HTML.
pub async fn handler() -> impl IntoResponse {
    IndexTemplate.render().map_or_else(
        |_| axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        |html| Html(html).into_response(),
    )
}
