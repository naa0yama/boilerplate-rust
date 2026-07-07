//! Static asset serving (CSS, JS, fonts).

use axum::body::Body;
use axum::http::{Response, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Router, extract::Path};

/// Compiled application CSS (`Tailwind` + `DaisyUI`).
pub static APP_CSS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/app.css"));

/// Bundled htmx JavaScript.
pub static HTMX_JS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/htmx.min.js"));

/// Embedded web fonts (woff2) from `$OUT_DIR/fonts`.
#[derive(rust_embed::RustEmbed)]
#[folder = "$OUT_DIR/fonts"]
pub struct Fonts;

impl std::fmt::Debug for Fonts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fonts").finish()
    }
}

fn ok_response(content_type: &'static str, body: Body) -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .body(body)
        .unwrap_or_else(|_| internal_error())
}

fn internal_error() -> Response<Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::empty())
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

async fn serve_css() -> impl IntoResponse {
    ok_response("text/css; charset=utf-8", Body::from(APP_CSS))
}

async fn serve_htmx() -> impl IntoResponse {
    ok_response("application/javascript; charset=utf-8", Body::from(HTMX_JS))
}

async fn serve_font(Path(path): Path<String>) -> impl IntoResponse {
    match Fonts::get(&path) {
        Some(file) => {
            let ext = std::path::Path::new(&path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            let mime = if ext.eq_ignore_ascii_case("woff2") {
                "font/woff2"
            } else {
                "application/octet-stream"
            };
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime)
                .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
                .body(Body::from(file.data.into_owned()))
                .unwrap_or_else(|_| internal_error())
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap_or_else(|_| internal_error()),
    }
}

/// Returns the router serving `/static/app.css`, `/static/htmx.min.js`, and `/fonts/{*path}`.
pub fn router() -> Router {
    Router::new()
        .route("/static/app.css", get(serve_css))
        .route("/static/htmx.min.js", get(serve_htmx))
        .route("/fonts/{*path}", get(serve_font))
}
