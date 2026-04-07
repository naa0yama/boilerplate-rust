//! HTTP client utilities with OTel metrics instrumentation.
//!
//! Demonstrates OTel HTTP client semantic conventions:
//! `http.client.request.duration` with `http.request.method`,
//! `http.response.status_code`, `server.address`, and `url.scheme` attributes.

use std::time::Instant;

use anyhow::Context as _;

use crate::metrics::Meters;

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
    use crate::metrics::Meters;

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
}
