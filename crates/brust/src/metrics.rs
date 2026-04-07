//! Application metric instruments.
//!
//! Create a [`Meters`] instance once via [`Meters::default`] after the global
//! `MeterProvider` has been initialized, then pass it by shared reference to
//! every function that records measurements.

// ---------------------------------------------------------------------------
// OTel implementation (feature = "otel")
// ---------------------------------------------------------------------------

#[cfg(feature = "process-metrics")]
mod process;

#[cfg(feature = "otel")]
use opentelemetry::metrics::{Counter, Histogram, UpDownCounter};
#[cfg(feature = "otel")]
use opentelemetry_semantic_conventions::{attribute, metric as semconv};

/// Collected `OTel` metric instruments for this application.
///
/// All instruments are created once and reused — do not construct per-request.
/// The `_process` field keeps observable process metric callbacks registered
/// for the lifetime of this struct (requires the `process-metrics` feature).
#[cfg(feature = "otel")]
pub struct Meters {
    // --- Sync instruments ---
    run_duration: Histogram<f64>,
    greeting_count: Counter<u64>,
    greeting_errors: Counter<u64>,
    iteration_count: Counter<u64>,
    iteration_duration: Histogram<f64>,
    iteration_in_flight: UpDownCounter<i64>,
    http_request_duration: Histogram<f64>,
    // --- Observable process metrics (feature = "process-metrics") ---
    // Disabled under Miri: sysinfo calls sysconf(_SC_CLK_TCK) which Miri does not stub.
    #[cfg(all(feature = "process-metrics", not(miri)))]
    _process: process::ProcessMetricHandles,
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
    /// Create all application instruments from the global `MeterProvider`.
    ///
    /// Call exactly once after `opentelemetry::global::set_meter_provider` has
    /// been called. When the `process-metrics` feature is enabled (default),
    /// process metric observable callbacks are also registered here.
    #[must_use]
    pub fn new() -> Self {
        let meter = opentelemetry::global::meter(env!("CARGO_PKG_NAME"));

        Self {
            run_duration: meter
                .f64_histogram("brust.run.duration")
                .with_unit("s")
                .with_description("End-to-end command execution latency")
                .build(),
            greeting_count: meter
                .u64_counter("brust.greeting.count")
                .with_unit("{call}")
                .with_description("Total greeting calls attributed by resolved gender")
                .build(),
            greeting_errors: meter
                .u64_counter("brust.greeting.errors")
                .with_unit("{error}")
                .with_description("Greeting calls that resulted in an error")
                .build(),
            iteration_count: meter
                .u64_counter("brust.iteration.count")
                .with_unit("{iter}")
                .with_description("Total iterations executed in the count demo")
                .build(),
            iteration_duration: meter
                .f64_histogram("brust.iteration.duration")
                .with_unit("s")
                .with_description("Per-iteration sleep delay in the count demo")
                .build(),
            iteration_in_flight: meter
                .i64_up_down_counter("brust.iteration.in_flight")
                .with_unit("{iter}")
                .with_description("Iterations currently executing (UpDownCounter demo)")
                .build(),
            http_request_duration: meter
                .f64_histogram(semconv::HTTP_CLIENT_REQUEST_DURATION)
                .with_unit("s")
                .with_description(
                    "HTTP client request duration including response body download \
                     (`OTel` HTTP semconv)",
                )
                .build(),
            #[cfg(all(feature = "process-metrics", not(miri)))]
            _process: process::ProcessMetricHandles::register(&meter),
        }
    }

    /// Record end-to-end command execution latency.
    ///
    /// `command` should be one of `"greet"`, `"count"`, or `"http"`.
    pub fn record_run_duration(&self, duration_s: f64, command: &str) {
        self.run_duration.record(
            duration_s,
            &[opentelemetry::KeyValue::new(
                "brust.command",
                command.to_owned(),
            )],
        );
    }

    /// Record a greeting call attributed by the resolved gender string.
    ///
    /// Low-cardinality values: `"man"`, `"woman"`, `"none"`, `"invalid"`.
    pub fn record_greeting(&self, gender: &str) {
        self.greeting_count.add(
            1,
            &[opentelemetry::KeyValue::new(
                "brust.gender",
                gender.to_owned(),
            )],
        );
    }

    /// Record a greeting error attributed by error type.
    ///
    /// Low-cardinality values: `"invalid_gender"`, `"unknown"`.
    pub fn record_greeting_error(&self, error_type: &str) {
        self.greeting_errors.add(
            1,
            &[opentelemetry::KeyValue::new(
                attribute::ERROR_TYPE,
                error_type.to_owned(),
            )],
        );
    }

    /// Record one completed iteration and its sleep duration in seconds.
    pub fn record_iteration(&self, duration_s: f64) {
        self.iteration_count.add(1, &[]);
        self.iteration_duration.record(duration_s, &[]);
    }

    /// Adjust the in-flight iteration counter by `delta` (`+1` start, `-1` end).
    pub fn in_flight_add(&self, delta: i64) {
        self.iteration_in_flight.add(delta, &[]);
    }

    /// Record an HTTP client request with `OTel` HTTP semantic convention attributes.
    ///
    /// - `method`: HTTP verb (`"GET"`, `"POST"`, …)
    /// - `status`: HTTP response status code
    /// - `host`: target host name
    /// - `scheme`: URL scheme (`"http"` or `"https"`)
    pub fn record_http_request(
        &self,
        duration_s: f64,
        method: &str,
        status: u16,
        host: &str,
        scheme: &str,
    ) {
        use opentelemetry::KeyValue;
        let attrs = [
            KeyValue::new(attribute::HTTP_REQUEST_METHOD, method.to_owned()),
            KeyValue::new(attribute::HTTP_RESPONSE_STATUS_CODE, i64::from(status)),
            KeyValue::new(attribute::SERVER_ADDRESS, host.to_owned()),
            KeyValue::new(attribute::URL_SCHEME, scheme.to_owned()),
        ];
        self.http_request_duration.record(duration_s, &attrs);
    }
}

// ---------------------------------------------------------------------------
// No-op stub (feature != "otel")
// ---------------------------------------------------------------------------

/// No-op metric instruments used when the `otel` feature is disabled.
#[cfg(not(feature = "otel"))]
#[derive(Debug, Default)]
pub struct Meters;

#[cfg(not(feature = "otel"))]
impl Meters {
    /// Record end-to-end command execution latency (no-op).
    pub fn record_run_duration(&self, _duration_s: f64, _command: &str) {}
    /// Record a greeting call (no-op).
    pub fn record_greeting(&self, _gender: &str) {}
    /// Record a greeting error (no-op).
    pub fn record_greeting_error(&self, _error_type: &str) {}
    /// Record one completed iteration (no-op).
    pub fn record_iteration(&self, _duration_s: f64) {}
    /// Adjust the in-flight counter (no-op).
    pub fn in_flight_add(&self, _delta: i64) {}
    /// Record an HTTP client request (no-op).
    pub fn record_http_request(
        &self,
        _duration_s: f64,
        _method: &str,
        _status: u16,
        _host: &str,
        _scheme: &str,
    ) {
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(all(test, feature = "otel"))]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use opentelemetry::metrics::MeterProvider as _;
    use opentelemetry_sdk::metrics::{
        InMemoryMetricExporter, SdkMeterProvider,
        data::{AggregatedMetrics, MetricData},
    };

    fn test_provider() -> (SdkMeterProvider, InMemoryMetricExporter) {
        let exporter = InMemoryMetricExporter::default();
        let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(exporter.clone()).build();
        let provider = SdkMeterProvider::builder().with_reader(reader).build();
        (provider, exporter)
    }

    /// Find a metric by name in the exported `ResourceMetrics` snapshot.
    fn find_metric<'a>(
        metrics: &'a [opentelemetry_sdk::metrics::data::ResourceMetrics],
        name: &str,
    ) -> Option<&'a opentelemetry_sdk::metrics::data::Metric> {
        metrics
            .iter()
            .flat_map(opentelemetry_sdk::metrics::data::ResourceMetrics::scope_metrics)
            .flat_map(opentelemetry_sdk::metrics::data::ScopeMetrics::metrics)
            .find(|m| m.name() == name)
    }

    #[test]
    fn greeting_count_increments_with_attributes() {
        use opentelemetry::KeyValue;

        let (provider, exporter) = test_provider();
        let meter = provider.meter("test");

        let greeting_count = meter
            .u64_counter("brust.greeting.count")
            .with_unit("{call}")
            .with_description("Total greeting calls attributed by resolved gender")
            .build();

        greeting_count.add(1, &[KeyValue::new("brust.gender", "man")]);
        greeting_count.add(1, &[KeyValue::new("brust.gender", "woman")]);
        greeting_count.add(1, &[KeyValue::new("brust.gender", "man")]);

        provider.force_flush().expect("flush failed");

        let metrics = exporter.get_finished_metrics().expect("no data");
        let metric =
            find_metric(&metrics, "brust.greeting.count").expect("brust.greeting.count not found");

        let total = match metric.data() {
            AggregatedMetrics::U64(MetricData::Sum(sum)) => sum
                .data_points()
                .map(opentelemetry_sdk::metrics::data::SumDataPoint::value)
                .sum::<u64>(),
            other => panic!("unexpected metric type: {other:?}"),
        };
        assert_eq!(total, 3);

        provider.shutdown().unwrap();
    }

    #[test]
    fn iteration_histogram_records_all_durations() {
        let (provider, exporter) = test_provider();
        let meter = provider.meter("test");

        let histogram = meter
            .f64_histogram("brust.iteration.duration")
            .with_unit("s")
            .with_description("Per-iteration sleep delay in the count demo")
            .build();

        histogram.record(1.0, &[]);
        histogram.record(3.0, &[]);
        histogram.record(5.0, &[]);

        provider.force_flush().expect("flush failed");

        let metrics = exporter.get_finished_metrics().expect("no data");
        let metric = find_metric(&metrics, "brust.iteration.duration")
            .expect("brust.iteration.duration not found");

        let (count, sum) = match metric.data() {
            AggregatedMetrics::F64(MetricData::Histogram(hist)) => {
                let dp = hist.data_points().next().expect("no data points");
                (dp.count(), dp.sum())
            }
            other => panic!("unexpected metric type: {other:?}"),
        };
        assert_eq!(count, 3);
        assert!((sum - 9.0).abs() < f64::EPSILON);

        provider.shutdown().unwrap();
    }

    #[test]
    fn in_flight_counter_tracks_net_change() {
        let (provider, exporter) = test_provider();
        let meter = provider.meter("test");

        let in_flight = meter
            .i64_up_down_counter("brust.iteration.in_flight")
            .with_unit("{iter}")
            .with_description("Iterations currently executing (UpDownCounter demo)")
            .build();

        in_flight.add(1, &[]);
        in_flight.add(1, &[]);
        in_flight.add(-1, &[]);

        provider.force_flush().expect("flush failed");

        let metrics = exporter.get_finished_metrics().expect("no data");
        let metric = find_metric(&metrics, "brust.iteration.in_flight")
            .expect("brust.iteration.in_flight not found");

        let value = match metric.data() {
            AggregatedMetrics::I64(MetricData::Sum(sum)) => sum
                .data_points()
                .map(opentelemetry_sdk::metrics::data::SumDataPoint::value)
                .sum::<i64>(),
            other => panic!("unexpected metric type: {other:?}"),
        };
        assert_eq!(value, 1);

        provider.shutdown().unwrap();
    }

    #[test]
    fn http_request_duration_records_semconv_attributes() {
        use opentelemetry::KeyValue;
        use opentelemetry_semantic_conventions::{attribute, metric as semconv};

        let (provider, exporter) = test_provider();
        let meter = provider.meter("test");

        let histogram = meter
            .f64_histogram(semconv::HTTP_CLIENT_REQUEST_DURATION)
            .with_unit("s")
            .with_description("HTTP client request duration including response body download")
            .build();

        let attrs = [
            KeyValue::new(attribute::HTTP_REQUEST_METHOD, "GET"),
            KeyValue::new(attribute::HTTP_RESPONSE_STATUS_CODE, 200_i64),
            KeyValue::new(attribute::SERVER_ADDRESS, "example.com"),
            KeyValue::new(attribute::URL_SCHEME, "https"),
        ];
        histogram.record(0.123, &attrs);

        provider.force_flush().expect("flush failed");

        let metrics = exporter.get_finished_metrics().expect("no data");
        let metric = find_metric(&metrics, semconv::HTTP_CLIENT_REQUEST_DURATION)
            .expect("http.client.request.duration not found");

        let (count, sum) = match metric.data() {
            AggregatedMetrics::F64(MetricData::Histogram(hist)) => {
                let dp = hist.data_points().next().expect("no data points");
                (dp.count(), dp.sum())
            }
            other => panic!("unexpected metric type: {other:?}"),
        };
        assert_eq!(count, 1);
        assert!((sum - 0.123).abs() < 1e-9);

        provider.shutdown().unwrap();
    }
}
