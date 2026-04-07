//! Brust - Rust ボイラープレートプロジェクト

/// ライブラリモジュール群
pub mod libs;
/// Application metric instruments
mod metrics;

use clap::Parser;
use tracing_subscriber::filter::EnvFilter;
#[cfg(not(feature = "otel"))]
use tracing_subscriber::fmt;
#[cfg(feature = "otel")]
use tracing_subscriber::layer::SubscriberExt;
#[cfg(feature = "otel")]
use tracing_subscriber::util::SubscriberInitExt;

use crate::libs::count;
use crate::libs::hello::{GreetingError, sayhello};
use crate::libs::http;
use crate::metrics::Meters;

#[derive(Parser)]
#[command(about, version = APP_VERSION)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value = "Youre")]
    name: String,
    /// Gender for greeting (man, woman)
    #[arg(short, long)]
    gender: Option<String>,
    /// Number of iterations to run with random delays (metrics demo)
    #[arg(short = 'c', long = "count")]
    count: Option<u32>,
    /// URL to fetch via HTTP GET (HTTP client metrics demo)
    #[arg(short = 'u', long = "url")]
    url: Option<String>,
}

const APP_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (rev:", env!("GIT_HASH"), ")",);

fn main() {
    // Install TLS crypto provider for reqwest (required by rustls-no-provider feature).
    // Ignored if a provider is already installed (e.g., across tests).
    let _ = rustls::crypto::ring::default_provider().install_default();

    #[cfg(not(feature = "otel"))]
    {
        fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
            )
            .init();
    }

    #[cfg(feature = "otel")]
    let otel_providers = init_otel();

    // Create metric instruments after the global MeterProvider is set up.
    let meters = Meters::default();

    let args = Args::parse();

    {
        // Root span wraps all command processing so child spans (run, run_count,
        // HTTP fetch) share a single trace_id and errors are captured in context.
        let root = tracing::info_span!("main");
        let _guard = root.enter();

        run(&args.name, args.gender.as_deref(), &meters);

        if let Some(count) = args.count {
            run_count(count, &meters);
        }

        if let Some(ref url) = args.url {
            let start = std::time::Instant::now();
            if let Err(e) = http::fetch_url(url, &meters) {
                tracing::error!("HTTP fetch failed: {e:#}");
            }
            meters.record_run_duration(start.elapsed().as_secs_f64(), "http");
        }
    } // _guard dropped here: root span exits before OTel shutdown

    #[cfg(feature = "otel")]
    shutdown_otel(otel_providers);
}

/// Providers returned by `OTel` initialization for shutdown.
#[cfg(feature = "otel")]
type OtelProviders = (
    Option<opentelemetry_sdk::trace::SdkTracerProvider>,
    Option<opentelemetry_sdk::metrics::SdkMeterProvider>,
    Option<opentelemetry_sdk::logs::SdkLoggerProvider>,
);

/// Build an `OTel` `Resource` for this process.
///
/// `OTEL_SERVICE_NAME` env var overrides the compiled-in package name.
#[cfg(feature = "otel")]
fn build_resource() -> opentelemetry_sdk::Resource {
    let service_name =
        std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| String::from(env!("CARGO_PKG_NAME")));
    opentelemetry_sdk::Resource::builder()
        .with_service_name(service_name)
        .with_attributes([
            opentelemetry::KeyValue::new(
                opentelemetry_semantic_conventions::attribute::SERVICE_VERSION,
                env!("CARGO_PKG_VERSION"),
            ),
            opentelemetry::KeyValue::new(
                opentelemetry_semantic_conventions::attribute::VCS_REF_HEAD_REVISION,
                env!("GIT_HASH"),
            ),
        ])
        .build()
}

// NOTEST(cfg): OTel init requires OTLP endpoint — covered by integration trace tests
/// Initialize `OTel` tracing, logging, and metrics providers.
#[cfg(feature = "otel")]
fn init_otel() -> OtelProviders {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,opentelemetry=off"));
    let fmt_layer = tracing_subscriber::fmt::layer();

    let (otel_trace_layer, tp, mp, lp, otel_log_layer) =
        std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .ok()
            .filter(|ep| !ep.is_empty())
            .and_then(|_| {
                let resource = build_resource();

                // --- Traces (batch: non-blocking, suitable for production) ---
                let span_exporter = opentelemetry_otlp::SpanExporter::builder()
                    .with_http()
                    .build()
                    .ok()?;
                let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
                    .with_resource(resource.clone())
                    .with_batch_exporter(span_exporter)
                    .build();
                let tracer = opentelemetry::trace::TracerProvider::tracer(
                    &tracer_provider,
                    env!("CARGO_PKG_NAME"),
                );
                let trace_layer = tracing_opentelemetry::layer().with_tracer(tracer);

                // --- Logs (batch: non-blocking) ---
                let log_exporter = opentelemetry_otlp::LogExporter::builder()
                    .with_http()
                    .build()
                    .ok()?;
                let logger_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
                    .with_resource(resource.clone())
                    .with_batch_exporter(log_exporter)
                    .build();
                let log_layer =
                    opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(
                        &logger_provider,
                    );

                // --- Metrics (PeriodicReader: exports every 5 s) ---
                let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
                    .with_http()
                    .build()
                    .ok()?;
                let metric_reader =
                    opentelemetry_sdk::metrics::PeriodicReader::builder(metric_exporter)
                        .with_interval(std::time::Duration::from_secs(5))
                        .build();
                let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
                    .with_resource(resource)
                    .with_reader(metric_reader)
                    .build();
                opentelemetry::global::set_meter_provider(meter_provider.clone());

                Some((
                    Some(trace_layer),
                    Some(tracer_provider),
                    Some(meter_provider),
                    Some(logger_provider),
                    Some(log_layer),
                ))
            })
            .unwrap_or((None, None, None, None, None));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(otel_trace_layer)
        .with(otel_log_layer)
        .init();

    (tp, mp, lp)
}

// NOTEST(cfg): OTel shutdown requires live providers — covered by integration trace tests
/// Shut down `OTel` providers in reverse initialization order.
#[cfg(feature = "otel")]
fn shutdown_otel((tracer_provider, meter_provider, logger_provider): OtelProviders) {
    if let Some(provider) = tracer_provider
        && let Err(e) = provider.shutdown()
    {
        tracing::warn!("failed to shutdown OTel tracer provider: {e}");
    }
    if let Some(provider) = meter_provider {
        if let Err(e) = provider.force_flush() {
            tracing::warn!("failed to flush OTel meter provider: {e}");
        }
        if let Err(e) = provider.shutdown() {
            tracing::warn!("failed to shutdown OTel meter provider: {e}");
        }
    }
    if let Some(provider) = logger_provider
        && let Err(e) = provider.shutdown()
    {
        tracing::warn!("failed to shutdown OTel logger provider: {e}");
    }
}

/// Run the greeting command and record `OTel` metrics.
///
/// # Arguments
/// * `name` - 挨拶対象の名前
/// * `gender` - 性別オプション（None, Some("man"), Some("woman"), その他）
/// * `meters` - Metric instruments; no-op when `otel` feature is disabled
#[cfg_attr(feature = "otel", tracing::instrument(skip(meters)))]
pub fn run(name: &str, gender: Option<&str>, meters: &Meters) {
    let start = std::time::Instant::now();
    let result = sayhello(name, gender);

    match &result {
        Ok(_) => meters.record_greeting(gender.unwrap_or("none")),
        Err(GreetingError::InvalidGender(_)) => {
            meters.record_greeting("invalid");
            meters.record_greeting_error("invalid_gender");
        }
        Err(GreetingError::UnknownGender) => {
            meters.record_greeting("none");
            meters.record_greeting_error("unknown");
        }
    }

    let greeting = format_greeting(name, result);
    tracing::info!("{}, new world!!", greeting);
    meters.record_run_duration(start.elapsed().as_secs_f64(), "greet");
}

/// Format a greeting from a `sayhello` result, handling errors gracefully.
fn format_greeting(name: &str, result: Result<String, GreetingError>) -> String {
    match result {
        Ok(msg) => msg,
        Err(GreetingError::InvalidGender(invalid_gender)) => {
            tracing::warn!(
                "Invalid gender '{}' specified, using default greeting",
                invalid_gender
            );
            format!("Hi, {name} (invalid gender: {invalid_gender})")
        }
        Err(GreetingError::UnknownGender) => {
            tracing::error!("Unexpected error in greeting generation, using default");
            format!("Hi, {name}")
        }
    }
}

/// Run iteration count demo and record `OTel` metrics.
#[cfg_attr(feature = "otel", tracing::instrument(skip(meters)))]
fn run_count(count: u32, meters: &Meters) {
    let start = std::time::Instant::now();
    meters.in_flight_add(1);

    let results = count::run_iterations(count);

    for result in &results {
        #[allow(clippy::cast_precision_loss, clippy::as_conversions)]
        // duration_secs (u64 1..=5) fits f64 losslessly
        meters.record_iteration(result.duration_secs as f64);
    }

    meters.in_flight_add(-1);
    meters.record_run_duration(start.elapsed().as_secs_f64(), "count");
}

#[cfg(test)]
mod tests {
    use super::{Meters, format_greeting, run};
    use crate::libs::hello::GreetingError;
    use tracing::subscriber::with_default;
    use tracing_mock::{expect, subscriber};

    /// Build a mock subscriber that expects the `run` instrumentation span
    /// wrapping a single event with the given message.
    fn mock_run_single_event(msg: &str) -> (impl tracing::Subscriber, subscriber::MockHandle) {
        let run_span = expect::span().named("run");
        subscriber::mock()
            .new_span(run_span.clone())
            .enter(run_span.clone())
            .event(expect::event().with_fields(expect::msg(msg)))
            .exit(run_span.clone())
            .drop_span(run_span)
            .only()
            .run_with_handle()
    }

    #[test]
    fn test_run_with_default_name() {
        let meters = Meters::default();
        let (subscriber, handle) = mock_run_single_event("Hi, Youre, new world!!");

        with_default(subscriber, || {
            run("Youre", None, &meters);
        });

        handle.assert_finished();
    }

    #[test]
    fn test_run_with_custom_name() {
        let meters = Meters::default();
        let (subscriber, handle) = mock_run_single_event("Hi, Alice, new world!!");

        with_default(subscriber, || {
            run("Alice", None, &meters);
        });

        handle.assert_finished();
    }

    #[test]
    fn test_run_with_empty_name() {
        let meters = Meters::default();
        let (subscriber, handle) = mock_run_single_event("Hi, , new world!!");

        with_default(subscriber, || {
            run("", None, &meters);
        });

        handle.assert_finished();
    }

    #[test]
    fn test_run_with_japanese_name() {
        let meters = Meters::default();
        let (subscriber, handle) = mock_run_single_event("Hi, 世界, new world!!");

        with_default(subscriber, || {
            run("世界", None, &meters);
        });

        handle.assert_finished();
    }

    #[test]
    fn test_run_with_gender_man() {
        let meters = Meters::default();
        let (subscriber, handle) = mock_run_single_event("Hi, Mr. John, new world!!");

        with_default(subscriber, || {
            run("John", Some("man"), &meters);
        });

        handle.assert_finished();
    }

    #[test]
    fn test_run_with_gender_woman() {
        let meters = Meters::default();
        let (subscriber, handle) = mock_run_single_event("Hi, Ms. Alice, new world!!");

        with_default(subscriber, || {
            run("Alice", Some("woman"), &meters);
        });

        handle.assert_finished();
    }

    #[test]
    fn test_format_greeting_unknown_gender() {
        let (subscriber, handle) = subscriber::mock()
            .event(
                expect::event()
                    .with_target(env!("CARGO_PKG_NAME"))
                    .at_level(tracing::Level::ERROR),
            )
            .only()
            .run_with_handle();

        with_default(subscriber, || {
            let result = format_greeting("Unknown", Err(GreetingError::UnknownGender));
            assert_eq!(result, "Hi, Unknown");
        });

        handle.assert_finished();
    }

    #[test]
    fn test_format_greeting_invalid_gender() {
        let (subscriber, handle) = subscriber::mock()
            .event(
                expect::event()
                    .with_target(env!("CARGO_PKG_NAME"))
                    .at_level(tracing::Level::WARN),
            )
            .only()
            .run_with_handle();

        with_default(subscriber, || {
            let result = format_greeting(
                "Bob",
                Err(GreetingError::InvalidGender(String::from("other"))),
            );
            assert_eq!(result, "Hi, Bob (invalid gender: other)");
        });

        handle.assert_finished();
    }

    #[test]
    fn test_format_greeting_ok() {
        let result = format_greeting("Alice", Ok(String::from("Hi, Alice")));
        assert_eq!(result, "Hi, Alice");
    }

    #[test]
    fn test_run_with_invalid_gender() {
        let run_span = expect::span().named("run");
        let (subscriber, handle) = subscriber::mock()
            .new_span(run_span.clone())
            .enter(run_span.clone())
            .event(
                expect::event()
                    .with_target(env!("CARGO_PKG_NAME"))
                    .at_level(tracing::Level::WARN),
            )
            .event(
                expect::event()
                    .with_fields(expect::msg("Hi, Bob (invalid gender: other), new world!!")),
            )
            .exit(run_span.clone())
            .drop_span(run_span)
            .only()
            .run_with_handle();

        let meters = Meters::default();
        with_default(subscriber, || {
            run("Bob", Some("other"), &meters);
        });

        handle.assert_finished();
    }
}
