//! Telemetry initialisation and shutdown.

pub mod metrics;

/// Guard that holds `OTel` providers for graceful shutdown.
///
/// Constructed by [`init_telemetry`] and consumed by [`TelemetryGuard::shutdown`].
#[allow(clippy::module_name_repetitions)]
pub enum TelemetryGuard {
    /// OTLP exporter is active; providers are held for graceful shutdown.
    #[cfg(feature = "otel")]
    Otlp {
        /// Tracer provider for `OTel` traces.
        tracer_provider: opentelemetry_sdk::trace::SdkTracerProvider,
        /// Meter provider for `OTel` metrics.
        meter_provider: opentelemetry_sdk::metrics::SdkMeterProvider,
        /// Logger provider for `OTel` logs.
        logger_provider: opentelemetry_sdk::logs::SdkLoggerProvider,
    },
    /// `OTel` is disabled (no `OTEL_EXPORTER_OTLP_ENDPOINT` set); no-op shutdown.
    Disabled,
}

impl std::fmt::Debug for TelemetryGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "otel")]
            Self::Otlp { .. } => f
                .debug_struct("TelemetryGuard::Otlp")
                .finish_non_exhaustive(),
            Self::Disabled => write!(f, "TelemetryGuard::Disabled"),
        }
    }
}

impl TelemetryGuard {
    /// Shut down all `OTel` providers in reverse initialisation order.
    pub fn shutdown(self) {
        #[cfg(feature = "otel")]
        if let Self::Otlp {
            tracer_provider,
            meter_provider,
            logger_provider,
        } = self
        {
            if let Err(e) = tracer_provider.shutdown() {
                tracing::warn!("failed to shutdown OTel tracer provider: {e}"); // NOTEST(unreachable): provider.shutdown() Err requires broken provider
            }
            if let Err(e) = meter_provider.force_flush() {
                tracing::warn!("failed to flush OTel meter provider: {e}"); // NOTEST(unreachable): provider.force_flush() Err requires broken provider
            }
            if let Err(e) = meter_provider.shutdown() {
                tracing::warn!("failed to shutdown OTel meter provider: {e}"); // NOTEST(unreachable): provider.shutdown() Err requires broken provider
            }
            if let Err(e) = logger_provider.shutdown() {
                tracing::warn!("failed to shutdown OTel logger provider: {e}"); // NOTEST(unreachable): provider.shutdown() Err requires broken provider
            }
        }
    }
}

/// Initialise `OTel` tracing, metrics, and logging providers.
///
/// Returns [`TelemetryGuard::Disabled`] when `OTEL_EXPORTER_OTLP_ENDPOINT` is unset.
/// The caller must call [`TelemetryGuard::shutdown`] before process exit.
///
/// # Errors
///
/// Returns an error if the tracing subscriber cannot be installed (e.g., called twice).
#[allow(clippy::module_name_repetitions)]
pub fn init_telemetry(
    service_name: &'static str,
    git_hash: &'static str,
) -> anyhow::Result<TelemetryGuard> {
    use tracing_subscriber::filter::EnvFilter;
    use tracing_subscriber::layer::SubscriberExt as _;
    use tracing_subscriber::util::SubscriberInitExt as _;

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,opentelemetry=off"));
    let fmt_layer = tracing_subscriber::fmt::layer();

    #[cfg(feature = "otel")]
    {
        let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .ok()
            .filter(|ep| !ep.is_empty());

        if let Some(_ep) = endpoint {
            let resource = build_resource(service_name, git_hash);

            // Traces
            let span_exporter = opentelemetry_otlp::SpanExporter::builder()
                .with_http()
                .build()
                .map_err(|e| anyhow::anyhow!("failed to build span exporter: {e}"))?;
            let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
                .with_resource(resource.clone())
                .with_batch_exporter(span_exporter)
                .build();
            opentelemetry::global::set_text_map_propagator(
                opentelemetry_sdk::propagation::TraceContextPropagator::new(),
            );
            opentelemetry::global::set_tracer_provider(tracer_provider.clone());
            let tracer =
                opentelemetry::trace::TracerProvider::tracer(&tracer_provider, service_name);
            let trace_layer = tracing_opentelemetry::layer().with_tracer(tracer);

            // Logs
            let log_exporter = opentelemetry_otlp::LogExporter::builder()
                .with_http()
                .build()
                .map_err(|e| anyhow::anyhow!("failed to build log exporter: {e}"))?;
            let logger_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
                .with_resource(resource.clone())
                .with_batch_exporter(log_exporter)
                .build();
            let log_layer = opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(
                &logger_provider,
            );

            // Metrics
            let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
                .with_http()
                .build()
                .map_err(|e| anyhow::anyhow!("failed to build metric exporter: {e}"))?;
            let metric_reader =
                opentelemetry_sdk::metrics::PeriodicReader::builder(metric_exporter)
                    .with_interval(std::time::Duration::from_secs(5))
                    .build();
            let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
                .with_resource(resource)
                .with_reader(metric_reader)
                .build();
            opentelemetry::global::set_meter_provider(meter_provider.clone());

            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .with(trace_layer)
                .with(log_layer)
                .try_init()
                .map_err(|e| anyhow::anyhow!("failed to init tracing subscriber: {e}"))?;

            return Ok(TelemetryGuard::Otlp {
                tracer_provider,
                meter_provider,
                logger_provider,
            });
        }
    }

    // Suppress unused-variable warnings in non-otel builds
    let _ = (service_name, git_hash);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()
        .map_err(|e| anyhow::anyhow!("failed to init tracing subscriber: {e}"))?;

    Ok(TelemetryGuard::Disabled)
}

/// Build an `OTel` `Resource` for this service.
#[cfg(feature = "otel")]
fn build_resource(
    service_name: &'static str,
    git_hash: &'static str,
) -> opentelemetry_sdk::Resource {
    use opentelemetry_semantic_conventions::attribute;
    opentelemetry_sdk::Resource::builder()
        .with_service_name(service_name)
        .with_attributes([
            opentelemetry::KeyValue::new(attribute::SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
            opentelemetry::KeyValue::new(
                attribute::SERVICE_INSTANCE_ID,
                gethostname::gethostname().to_string_lossy().into_owned(),
            ),
            opentelemetry::KeyValue::new(attribute::VCS_REF_HEAD_REVISION, git_hash),
        ])
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_debug_format() {
        assert_eq!(
            format!("{:?}", TelemetryGuard::Disabled),
            "TelemetryGuard::Disabled",
        );
    }

    #[test]
    fn disabled_shutdown_no_panic() {
        TelemetryGuard::Disabled.shutdown();
    }

    #[cfg(feature = "otel")]
    #[test]
    fn otlp_debug_format() {
        use opentelemetry_sdk::logs::SdkLoggerProvider;
        use opentelemetry_sdk::metrics::SdkMeterProvider;
        use opentelemetry_sdk::trace::SdkTracerProvider;

        let guard = TelemetryGuard::Otlp {
            tracer_provider: SdkTracerProvider::builder().build(),
            meter_provider: SdkMeterProvider::builder().build(),
            logger_provider: SdkLoggerProvider::builder().build(),
        };
        let s = format!("{guard:?}");
        assert!(s.contains("TelemetryGuard::Otlp"), "unexpected: {s}");
    }

    #[cfg(feature = "otel")]
    #[test]
    fn otlp_shutdown_no_panic() {
        use opentelemetry_sdk::logs::SdkLoggerProvider;
        use opentelemetry_sdk::metrics::SdkMeterProvider;
        use opentelemetry_sdk::trace::SdkTracerProvider;

        let guard = TelemetryGuard::Otlp {
            tracer_provider: SdkTracerProvider::builder().build(),
            meter_provider: SdkMeterProvider::builder().build(),
            logger_provider: SdkLoggerProvider::builder().build(),
        };
        guard.shutdown();
    }
}
