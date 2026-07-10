//! GET / — index page.

use askama::Template;
use axum::response::{Html, IntoResponse};

#[derive(Template)]
#[template(path = "pages/index.html")]
struct IndexTemplate;

/// Renders the index page as HTML.
pub async fn handler() -> impl IntoResponse {
    let mut buf = String::new();
    IndexTemplate.render_into(&mut buf).map_or_else(
        |_| axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(), // NOTEST(unreachable): Askama render_into() never fails for a compile-validated template
        |()| Html(buf).into_response(),
    )
}
