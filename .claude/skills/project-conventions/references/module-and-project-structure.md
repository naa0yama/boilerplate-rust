# Module & Project Structure — boilerplate-rust

> **Shared patterns**: See `~/.claude/skills/rust-project-conventions/references/module-structure.md`
> for visibility rules, mod.rs re-export pattern, size limits, CLI design, and clippy configuration.

## Project Source Layout

```
src/
  main.rs              # CLI entry point (clap derive)
  libs.rs              # Top-level library module
  libs/
    hello.rs           # Greeting logic (example module)
tests/
  integration_test.rs  # Integration tests (assert_cmd)
ast-rules/
  *.yml                # Custom ast-grep lint rules
```

## OTel / Tracing Setup

- OTel is enabled by default (`default = ["otel", "process-metrics"]`).
- Set `OTEL_EXPORTER_OTLP_ENDPOINT` env var to activate OTLP export.
- Without the env var (or empty), only the `fmt` layer is active.
- Build without OTel: `mise run build -- --no-default-features`.
- Test tasks automatically set `OTEL_EXPORTER_OTLP_ENDPOINT=""` to prevent OTel panics.
- Feature flags in `Cargo.toml`:
  ```toml
  [features]
  default = ["otel", "process-metrics"]
  otel = [
  	"dep:gethostname",
  	"dep:opentelemetry",
  	"dep:opentelemetry_sdk",
  	"dep:opentelemetry-otlp",
  	"dep:tracing-opentelemetry",
  	"dep:opentelemetry-appender-tracing",
  	"dep:opentelemetry-semantic-conventions",
  ]
  # Collects OTel-semconv process metrics. Requires `otel`. Disable with --no-default-features.
  process-metrics = [
  	"otel",
  	"dep:sysinfo",
  ]
  ```
- `service.instance.id` is set to `gethostname::gethostname()` (CLI: one instance per host).
- `TraceContextPropagator` and `global::set_tracer_provider()` are set at provider init.
- Transport: HTTP/proto (`http-proto` + `reqwest-blocking-client`), port 4318.
