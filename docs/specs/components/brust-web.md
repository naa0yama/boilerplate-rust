# brust-web Crate Specification

## Overview

`brust-web` is a minimal web server boilerplate crate within the `boilerplate-rust` workspace.
It demonstrates Axum + Askama + DaisyUI (pnpm/Tailwind CSS) + OTel 3-signal instrumentation.

## Crate Responsibilities

- Serve an HTML page via Askama templates (DaisyUI styled)
- Provide a health check endpoint (`GET /health`)
- Serve embedded static assets (CSS, JS, fonts)
- Initialize and shut down OTel telemetry (traces / metrics / logs)
- Expose a CLI with `serve` and `version` subcommands

## Routing

| Method | Path             | Handler                   | Description               |
| ------ | ---------------- | ------------------------- | ------------------------- |
| GET    | `/`              | `routes::index::handler`  | Renders index page (HTML) |
| GET    | `/health`        | `routes::health::handler` | Returns `{"status":"ok"}` |
| GET    | `/static/*`      | `assets::router`          | app.css, htmx.min.js      |
| GET    | `/fonts/{*path}` | `assets::router`          | IBM Plex fonts (woff2)    |

## Static Assets

Assets are embedded at build time via `rust-embed` and `include_bytes!`:

- `app.css` — compiled from `src/styles/app.css` by Tailwind CSS / DaisyUI via pnpm
- `htmx.min.js` — copied from `node_modules/htmx.org/dist/`
- `fonts/` — IBM Plex Sans JP and IBM Plex Mono woff2 files from `@fontsource` packages

Font responses include `Cache-Control: public, max-age=31536000, immutable`.

## Template Structure

Four-directory Askama layout:

```
templates/
  shells/     # L1: HTML chrome — minimal.html (html/head/navbar)
  pages/      # L2: Page content — extends shells/minimal.html
  components/ # L3: Reusable macros — welcome_card.html
  partials/   # L4: HTMX fragments (reserved, currently empty)
```

## CLI Subcommands

| Subcommand       | Behavior                                         |
| ---------------- | ------------------------------------------------ |
| `serve [--bind]` | Start Axum HTTP server (default: `0.0.0.0:3000`) |
| `version`        | Print crate version from `CARGO_PKG_VERSION`     |

## build.rs Behavior

1. Checks for `CSS_PIPELINE_STUB=1` env var or pnpm absence → stub mode (empty assets)
2. Runs `pnpm install --frozen-lockfile`
3. Compiles `src/styles/app.css` → `$OUT_DIR/app.css` via Tailwind CSS CLI
4. Registers Askama template files for `cargo:rerun-if-changed`
5. Copies font woff2 files to `$OUT_DIR/fonts/<family>/`
6. Copies `htmx.min.js` to `$OUT_DIR/htmx.min.js`
7. Embeds `GIT_HASH` via `cargo:rustc-env`

Error type: `Box<dyn Error>` (not `anyhow`).

## Module Layout

```
src/
  main.rs       — tokio::main, CLI dispatch, OTel init/shutdown
  lib.rs        — pub mod declarations, app_version()
  assets.rs     — rust-embed static asset router
  cli.rs        — Clap CLI (Cli / Commands / ServeArgs)
  trace.rs      — OtelHttpServerMakeSpan, OtelOnResponse, server_metrics_mw
  telemetry/
    mod.rs      — TelemetryGuard, init_telemetry, shutdown
    metrics.rs  — Meters (http.server.request.duration histogram)
  routes/
    mod.rs
    index.rs    — GET /
    health.rs   — GET /health
```

## OTel Instrumentation

See [otel-instrumentation.md](../architecture/otel-instrumentation.md) for the full
3-signal specification.
