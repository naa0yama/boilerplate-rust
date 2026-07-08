//! HTTP client utilities with OTel metrics instrumentation.
//!
//! Demonstrates OTel HTTP client semantic conventions:
//! `http.client.request.duration` with `http.request.method`,
//! `http.response.status_code`, `server.address`, and `url.scheme` attributes.

use std::time::Instant;

use anyhow::Context as _;

use crate::telemetry::metrics::Meters;

/// Perform an HTTP GET request to `url` and record `OTel` client metrics.
///
/// Records `http.client.request.duration` with `OTel` HTTP semantic convention
/// attributes. The duration covers the full round-trip including response body
/// download.
///
/// # Errors
///
/// Returns an error if the URL is invalid, the TCP connection fails, the server
/// returns a network-level error, or the response body cannot be read.
#[cfg_attr(
    feature = "otel",
    tracing::instrument(
        skip(meters),
        fields(otel.kind = ?opentelemetry::trace::SpanKind::Client)
    )
)]
pub fn fetch_url(url: &str, meters: &Meters) -> anyhow::Result<()> {
    let parsed = reqwest::Url::parse(url).context("invalid URL")?;
    let host = parsed.host_str().unwrap_or("unknown").to_owned();
    let scheme = parsed.scheme().to_owned();

    let client = reqwest::blocking::Client::new();

    let start = Instant::now();
    let response = client.get(url).send().context("HTTP request failed")?;
    let status = response.status().as_u16();
    let _ = response.bytes().context("failed to read response body")?;
    let duration_s = start.elapsed().as_secs_f64();

    meters.record_http_request(duration_s, "GET", status, &host, &scheme);

    tracing::info!(
        http.request.method = "GET",
        http.response.status_code = status,
        server.address = %host,
        url.scheme = %scheme,
        duration_s,
        "HTTP GET completed",
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::metrics::Meters;

    #[test]
    fn fetch_url_rejects_invalid_url() {
        let meters = Meters::default();
        let result = fetch_url("not-a-url", &meters);
        assert!(result.is_err(), "expected error for invalid URL");
    }

    #[test]
    fn fetch_url_rejects_empty_url() {
        let meters = Meters::default();
        let result = fetch_url("", &meters);
        assert!(result.is_err(), "expected error for empty URL");
    }

    #[tokio::test]
    async fn fetch_url_success_records_metrics() {
        use axum::{Router, routing::get};
        use std::net::SocketAddr;

        let _ = rustls::crypto::ring::default_provider().install_default();

        let app = Router::new().route("/", get(|| async { "ok" }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind failed");
        let addr: SocketAddr = listener.local_addr().expect("local_addr failed");
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("server error");
        });

        let url = format!("http://127.0.0.1:{}/", addr.port());
        let meters = Meters::default();
        let result = tokio::task::spawn_blocking(move || fetch_url(&url, &meters))
            .await
            .expect("spawn_blocking panicked");
        assert!(result.is_ok(), "expected Ok for successful GET: {result:?}");
    }

    #[tokio::test]
    async fn fetch_url_connection_refused_returns_error() {
        let _ = rustls::crypto::ring::default_provider().install_default();

        // Bind and immediately drop to get a port that is not listening.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind failed");
        let port = listener.local_addr().expect("local_addr").port();
        drop(listener);

        let url = format!("http://127.0.0.1:{port}/");
        let meters = Meters::default();
        let result = tokio::task::spawn_blocking(move || fetch_url(&url, &meters))
            .await
            .expect("spawn_blocking panicked");
        assert!(result.is_err(), "expected error for connection refused");
    }
}
