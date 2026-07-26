# Testing & Coverage Strategy

## Coverage Target

- CI hard floor: 90% (`mise run pre-push` fails under `--fail-under-lines 90`).
- Working target: 100% line/branch coverage. `#[coverage(off)]` is not used
  to reach the floor — every branch gets a real test or an explicit
  `// NOTEST(unreachable): <why>` comment (see
  `~/.claude/skills/rust-implementation/references/coverage-tooling.md`
  "Coverage Target Strategy").
- `fn main` / CLI entrypoints are covered via integration tests that spawn
  the actual binary (`assert_cmd::Command::cargo_bin`) rather than excluded
  from coverage.

## External Dependencies in Tests

OTLP receivers and HTTP servers exercised in tests run as in-process fakes
(`tokio::net::TcpListener::bind("127.0.0.1:0")` + a tiny axum handler)
instead of a real OTLP collector or wiremock — this avoids external
dependencies and keeps CI stable. Considered alternatives (wiremock, a real
`o2` instance) were rejected for the same reason; not ADR-worthy since the
approach is reversible and consistent with the existing `tests/` pattern.

## brust / brust-web Coverage History

Baseline on `feat/wireframe-daisyui-thumbnails` (2026-07-08): 74.16%
(999/1347 lines). Brought to 97.01% via TDD cycles targeting each
uncovered file (`telemetry/mod.rs`, `main.rs` Serve path, `assets.rs`,
`trace.rs` branch coverage, `libs/http.rs` fetch success path, `main.rs`
Run path, residual `NOTEST` cleanup). Full cycle-by-cycle breakdown lived
in the now-absorbed `2026-07-08-test-coverage-100.md` design spec.

Final state: 97.01% (90% floor cleared by +7.01%). The remaining 2.99%
(~40 lines) is 22 `NOTEST(unreachable)` sites where adding a test would
require mocking the OTel SDK for near-zero benefit:

- `Response::builder` static status/empty-body `Err` paths
- OTel provider shutdown/force_flush `Err` paths
- Exhaustive `match` guards over OTel SDK metric types
- `signal()` under `cfg(not(unix))` (non-Unix build target, not exercised
  in CI)
- Askama `render()` `Err` path
- Internal test-server panic path
- CLI exhaustive-guard branches unreachable via public API

## Known Deviation: Production Code Changed for Test Enablement

The Cycle A `main.rs` Serve-path integration test required `main.rs` to log
its resolved `local_addr` (so the test can parse the actually-bound port
when `--bind` uses port `0`) and to handle `SIGTERM`/`SIGINT` for graceful
shutdown at test teardown. This was production code changed to make a test
possible, not test code alone — both changes are reversible (removing the
log line or signal handling doesn't affect runtime correctness) and are now
part of the crate's documented CLI behavior (see
`docs/specs/components/brust-web.md` CLI Subcommands section).
