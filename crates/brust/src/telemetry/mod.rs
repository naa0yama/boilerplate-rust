//! OpenTelemetry instrumentation root module.
//!
//! Hosts cross-signal conventions and signal-specific submodules.
//! Add `tracing` / `logs` submodules here when adopting those signals.

#[cfg(feature = "otel")]
pub mod conventions;

pub mod metrics;
