//! `OTel` HTTP server tracing middleware.

use std::sync::Arc;
use std::time::Instant;

use axum::{
    extract::{MatchedPath, Request, State},
    middleware::Next,
    response::Response,
};
use tower_http::trace::{MakeSpan, OnResponse};
use tracing::Span;

#[cfg(feature = "otel")]
use opentelemetry::{global, propagation::Extractor};
#[cfg(feature = "otel")]
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

use axum::http;

use crate::telemetry::metrics::Meters;

// ---------------------------------------------------------------------------
// W3C header extractor (otel feature only)
// ---------------------------------------------------------------------------

#[cfg(feature = "otel")]
struct HeaderExtractor<'a>(&'a http::HeaderMap);

#[cfg(feature = "otel")]
impl Extractor for HeaderExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(http::HeaderName::as_str).collect()
    }
}

// ---------------------------------------------------------------------------
// MakeSpan
// ---------------------------------------------------------------------------

/// Creates `OTel` `SERVER` spans for incoming HTTP requests.
///
/// Sets `otel.name` to `"{METHOD} {route}"` and extracts W3C `traceparent`
/// when the `otel` feature is enabled.
#[derive(Clone, Debug)]
pub struct OtelHttpServerMakeSpan;

impl<B> MakeSpan<B> for OtelHttpServerMakeSpan {
    fn make_span(&mut self, request: &http::Request<B>) -> Span {
        let method = request.method().as_str();
        let route = request
            .extensions()
            .get::<MatchedPath>()
            .map_or_else(|| request.uri().path(), |p| p.as_str());

        let scheme = request.uri().scheme_str().unwrap_or("http");
        let path = request.uri().path();
        let query = request.uri().query().unwrap_or("");
        let full_url = format!("{scheme}://localhost{path}");
        let user_agent = request
            .headers()
            .get(http::header::USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let network_protocol = match request.version() {
            http::Version::HTTP_09 => "0.9",
            http::Version::HTTP_2 => "2",
            http::Version::HTTP_3 => "3",
            _ => "1.1",
        };

        let span_name = format!("{method} {route}");
        let span = tracing::info_span!(
            "http_request",
            otel.name = %span_name,
            otel.kind = "server",
            "http.request.method" = method,
            "url.scheme" = scheme,
            "url.path" = path,
            "url.query" = query,
            "url.full" = %full_url,
            "http.route" = route,
            "user_agent.original" = user_agent,
            "network.protocol.version" = network_protocol,
            "http.response.status_code" = tracing::field::Empty,
            "error.type" = tracing::field::Empty,
        );

        #[cfg(feature = "otel")]
        {
            let extractor = HeaderExtractor(request.headers());
            let parent_ctx = global::get_text_map_propagator(|p| p.extract(&extractor));
            let _ = span.set_parent(parent_ctx);
        }

        span
    }
}

// ---------------------------------------------------------------------------
// OnResponse
// ---------------------------------------------------------------------------

/// Records `http.response.status_code` on the current span.
///
/// Sets span status to `Error` for 5xx responses.
#[derive(Clone, Debug)]
pub struct OtelOnResponse;

impl<B> OnResponse<B> for OtelOnResponse {
    fn on_response(self, response: &http::Response<B>, _latency: std::time::Duration, span: &Span) {
        let status = response.status().as_u16();
        span.record("http.response.status_code", status);

        if status >= 500 {
            let error_type = status.to_string();
            span.record("error.type", error_type.as_str());
            #[cfg(feature = "otel")]
            span.set_status(opentelemetry::trace::Status::error(format!(
                "HTTP {status}"
            )));
        }
    }
}

// ---------------------------------------------------------------------------
// server_metrics_mw
// ---------------------------------------------------------------------------

/// Axum middleware that records `http.server.request.duration` via [`Meters`].
pub async fn server_metrics_mw(
    State(meters): State<Arc<Meters>>,
    matched_path: Option<MatchedPath>,
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let route = matched_path.map_or_else(
        || request.uri().path().to_owned(),
        |p| p.as_str().to_owned(),
    );

    let response = next.run(request).await;

    let duration_s = start.elapsed().as_secs_f64();
    let status = response.status().as_u16();
    let error_type = if status >= 500 {
        Some(status.to_string())
    } else {
        None
    };

    meters.record_http_server_request(
        duration_s,
        method.as_str(),
        status,
        &route,
        error_type.as_deref(),
    );

    response
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use axum::body::Body;
    use http::{Method, Request};

    #[test]
    fn make_span_does_not_panic_on_minimal_request() {
        let mut maker = OtelHttpServerMakeSpan;
        let req = Request::builder()
            .method(Method::GET)
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let _span = maker.make_span(&req);
    }

    #[test]
    fn make_span_does_not_panic_with_query_and_user_agent() {
        let mut maker = OtelHttpServerMakeSpan;
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/items?foo=bar")
            .header("user-agent", "test-client/1.0")
            .body(Body::empty())
            .unwrap();
        let _span = maker.make_span(&req);
    }

    #[tokio::test]
    async fn server_metrics_mw_records_without_panic() {
        use axum::{Router, routing::get};
        use tower::ServiceExt;

        #[cfg(feature = "otel")]
        {
            let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder().build();
            opentelemetry::global::set_meter_provider(provider);
        }

        let meters = Arc::new(Meters::new());
        let app: Router = Router::new()
            .route("/health", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn_with_state(
                Arc::clone(&meters),
                server_metrics_mw,
            ));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status().as_u16(), 200);
    }
}
