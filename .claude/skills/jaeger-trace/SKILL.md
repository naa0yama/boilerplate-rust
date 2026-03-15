---
name: jaeger-trace
description: >-
  Analyze Jaeger traces for debugging test failures and performance issues.
  Queries the local Jaeger v3 API (OTLP-based JSON) to retrieve recent traces,
  identify error spans, and surface slow operations. Use when: (1) tests fail
  with `mise run test:trace` and you need root-cause analysis via distributed
  traces, (2) investigating performance issues or unexpected latency,
  (3) verifying that spans and instrumentation are emitted correctly.
  Jaeger is auto-started by `mise run test:trace`; for manual use, run
  `mise run jaeger` first.
---

# Jaeger Trace Analysis

## Service Name

Read `Cargo.toml` `package.name` — the OTel SDK uses `CARGO_PKG_NAME` as the service name.

## API Reference (v3 — Stable)

Base URL: `http://localhost:16686` (Caddy reverse proxy).

```
GET /api/v3/services
GET /api/v3/operations?service={service}
GET /api/v3/traces?query.service_name={service}&query.num_traces=20&query.start_time_min={RFC3339}&query.start_time_max={RFC3339}
GET /api/v3/traces/{traceID}
```

Time parameters use RFC 3339 format. Generate them with:

```bash
START=$(date -u -d '10 minutes ago' +%Y-%m-%dT%H:%M:%SZ)
END=$(date -u +%Y-%m-%dT%H:%M:%SZ)
```

## Response Format (OTLP)

All trace endpoints return OTLP JSON:

```json
{
  "result": {
    "resourceSpans": [{
      "resource": { "attributes": [{"key": "service.name", "value": {"stringValue": "..."}}] },
      "scopeSpans": [{
        "spans": [{
          "traceId": "hex",
          "spanId": "hex",
          "parentSpanId": "hex",
          "name": "operation_name",
          "kind": 1,
          "startTimeUnixNano": "...",
          "endTimeUnixNano": "...",
          "attributes": [{"key": "...", "value": {...}}],
          "status": {"code": 2, "message": "error details"}
        }]
      }]
    }]
  }
}
```

Key fields for error detection:

- `status.code`: `2` = ERROR, `1` = OK, `0` = UNSET
- `attributes` with `key: "exception.message"` or `key: "exception.stacktrace"`
- Duration: `(endTimeUnixNano - startTimeUnixNano) / 1_000_000` ms

## Workflow

1. **Check Jaeger availability**
   ```bash
   curl -s http://localhost:16686/api/v3/services
   ```
   If this fails, tell the user to run `mise run jaeger`.

2. **Fetch recent traces**
   ```bash
   curl -s "http://localhost:16686/api/v3/traces?query.service_name={service}&query.num_traces=20&query.start_time_min=${START}&query.start_time_max=${END}"
   ```
   Widen time range if no traces found.

3. **Analyze traces** — look for:
   - Error spans: `status.code == 2`
   - Exception attributes: `exception.message`, `exception.stacktrace`
   - Slow spans: compare `endTimeUnixNano - startTimeUnixNano` across siblings

4. **Drill into specific trace** (if needed)
   ```bash
   curl -s "http://localhost:16686/api/v3/traces/{traceID}"
   ```

5. **Report**: trace count, error spans (operation, message, traceID), slow spans (operation, duration, traceID), and suggested next steps.
