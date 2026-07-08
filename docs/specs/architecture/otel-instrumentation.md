# OTel Instrumentation Specification

## Overview

`brust-web` implements OpenTelemetry 3-signal instrumentation (traces, metrics, logs)
via OTLP export. When `OTEL_EXPORTER_OTLP_ENDPOINT` is unset, telemetry is disabled
and only console logging is active.

## Initialization

`telemetry::init_telemetry(service_name, git_hash) -> anyhow::Result<TelemetryGuard>`

Returns `TelemetryGuard::Otlp { tracer_provider, meter_provider, logger_provider }`
when OTLP endpoint is configured, otherwise `TelemetryGuard::Disabled`.

### Resource Attributes

| Attribute               | Value                        |
| ----------------------- | ---------------------------- |
| `service.name`          | `CARGO_PKG_NAME`             |
| `service.version`       | `CARGO_PKG_VERSION`          |
| `service.instance.id`   | hostname (via `gethostname`) |
| `vcs.ref.head.revision` | `GIT_HASH` (from build.rs)   |

### Exporters

| Signal  | Exporter           | Configuration                |
| ------- | ------------------ | ---------------------------- |
| Traces  | OTLP gRPC batch    | `TraceContextPropagator`     |
| Metrics | OTLP gRPC periodic | 5 s interval                 |
| Logs    | OTLP gRPC batch    | `OpenTelemetryTracingBridge` |

### Shutdown Order

`tracer_provider` → `meter_provider` (force_flush then shutdown) → `logger_provider`

## Traces

**SpanKind:** `SERVER`

**Span name format:** `{METHOD} {route}` (e.g., `GET /`, `GET /health`)

**Attributes:**

| Attribute                   | Source                    |
| --------------------------- | ------------------------- |
| `http.request.method`       | Request method            |
| `url.scheme`                | `http` or `https`         |
| `url.path`                  | Request URI path          |
| `url.query`                 | Request URI query string  |
| `url.full`                  | Full request URI          |
| `http.route`                | Matched route pattern     |
| `user_agent.original`       | `User-Agent` header       |
| `network.protocol.version`  | HTTP version              |
| `server.address`            | Server host               |
| `client.address`            | Remote peer address       |
| `http.response.status_code` | Response status code      |
| `error.type`                | HTTP status string on 5xx |

**W3C TraceContext:** `traceparent` header extracted by `OtelHttpServerMakeSpan`.

**Error recording:** `OtelOnResponse` sets span status to `Error` and records
`error.type` on 5xx responses.

## Metrics

### `http.server.request.duration`

| Property    | Value                                    |
| ----------- | ---------------------------------------- |
| Instrument  | Histogram                                |
| Unit        | `s` (seconds)                            |
| Description | Duration of HTTP server request handling |

**Attributes:**

| Attribute                   | Description          |
| --------------------------- | -------------------- |
| `http.request.method`       | HTTP method          |
| `http.route`                | Matched route        |
| `http.response.status_code` | Response status code |
| `error.type`                | Set on 5xx only      |

**Explicit bucket boundaries (semconv):**
`[0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1.0, 2.5, 5.0, 7.5, 10.0]`

## Logs

Logs are emitted via `tracing` macros and bridged to OTel via
`OpenTelemetryTracingBridge`. When OTLP endpoint is not configured,
logs are written to stdout only.

## Feature Flags

| Feature           | Enables                                                   |
| ----------------- | --------------------------------------------------------- |
| `otel` (default)  | All OTel providers, propagators, OTLP exporters           |
| `process-metrics` | Adds `sysinfo` for process-level metrics (implies `otel`) |
