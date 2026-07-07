//! HTTP server request duration metrics.

#[cfg(feature = "otel")]
use opentelemetry::metrics::Histogram;
#[cfg(feature = "otel")]
use opentelemetry_semantic_conventions::{attribute, metric as semconv};

/// Collected `OTel` metric instruments for HTTP server observability.
#[cfg(feature = "otel")]
pub struct Meters {
    server_request_duration: Histogram<f64>,
}

#[cfg(feature = "otel")]
impl std::fmt::Debug for Meters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Meters").finish_non_exhaustive()
    }
}

#[cfg(feature = "otel")]
impl Default for Meters {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "otel")]
impl Meters {
    /// Create all HTTP server instruments from the global `MeterProvider`.
    ///
    /// Call exactly once after `opentelemetry::global::set_meter_provider` has been called.
    #[must_use]
    pub fn new() -> Self {
        // semconv bucket boundaries for http.server.request.duration
        let boundaries = vec![
            0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1.0, 2.5, 5.0, 7.5, 10.0,
        ];
        let meter = opentelemetry::global::meter(env!("CARGO_PKG_NAME"));
        Self {
            server_request_duration: meter
                .f64_histogram(semconv::HTTP_SERVER_REQUEST_DURATION)
                .with_unit("s")
                .with_description("Duration of HTTP server requests (`OTel` HTTP semconv)")
                .with_boundaries(boundaries)
                .build(),
        }
    }

    /// Record an HTTP server request with `OTel` HTTP semantic convention attributes.
    pub fn record_http_server_request(
        &self,
        duration_s: f64,
        method: &str,
        status: u16,
        route: &str,
        error_type: Option<&str>,
    ) {
        use opentelemetry::KeyValue;
        let mut attrs = vec![
            KeyValue::new(attribute::HTTP_REQUEST_METHOD, method.to_owned()),
            KeyValue::new(attribute::HTTP_RESPONSE_STATUS_CODE, i64::from(status)),
            KeyValue::new(attribute::HTTP_ROUTE, route.to_owned()),
        ];
        if let Some(et) = error_type {
            attrs.push(KeyValue::new(attribute::ERROR_TYPE, et.to_owned()));
        }
        self.server_request_duration.record(duration_s, &attrs);
    }
}

/// No-op metric instruments used when the `otel` feature is disabled.
#[cfg(not(feature = "otel"))]
#[derive(Debug, Default)]
pub struct Meters;

#[cfg(not(feature = "otel"))]
impl Meters {
    /// No-op constructor.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Record an HTTP server request (no-op).
    pub fn record_http_server_request(
        &self,
        _duration_s: f64,
        _method: &str,
        _status: u16,
        _route: &str,
        _error_type: Option<&str>,
    ) {
    }
}

#[cfg(all(test, feature = "otel"))]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use opentelemetry_sdk::metrics::SdkMeterProvider;

    #[test]
    fn meters_new_does_not_panic() {
        let provider = SdkMeterProvider::builder().build();
        opentelemetry::global::set_meter_provider(provider);
        let _meters = Meters::new();
    }
}
